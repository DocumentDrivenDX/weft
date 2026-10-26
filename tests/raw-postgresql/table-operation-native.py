# SPDX-License-Identifier: MIT OR Apache-2.0
"""Synthetic PG18 SQL oracle, not a production operation executor or authority.

The source is approved/installed by public UMF APIs. Tests deliberately execute
candidate SQL as ordinary roles inside caller transactions, check its declared
bounds and roll back faults. Durable data replay, opaque cursor authentication
and generated runtime admission remain unqualified.
"""
import argparse
import asyncio
import copy
from contextlib import contextmanager, asynccontextmanager
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import re
import sys
import time
import uuid

import psycopg
from psycopg.conninfo import conninfo_to_dict, make_conninfo
from psycopg import sql
import weft
from umf_codegen import compile_table_deployment, compile_table_migration
from umf_access.lifecycle import apply_table_migration, inspect_table_history
from umf_access.unified import SyncSession, AsyncSession, SessionIdentity, SessionPolicy

parser=argparse.ArgumentParser()
parser.add_argument('--dsn')
parser.add_argument('--dsn-env')
parser.add_argument('--report',required=True,type=Path)
args=parser.parse_args()
if bool(args.dsn) == bool(args.dsn_env): parser.error('provide exactly one of --dsn or --dsn-env')
if args.dsn_env:
    args.dsn=os.environ.get(args.dsn_env)
    if not args.dsn: parser.error('DSN environment variable is empty')
native_deadline=time.monotonic()+max(0,int(os.environ.get('WEFT_NATIVE_TIMEOUT_SECONDS','300'))-5)
def _bounded_timeout_ms(value,cap_ms,name):
    match=re.fullmatch(r'([0-9]+)(ms|s|min|h|d)?',value.strip(),re.IGNORECASE)
    if match is None:raise ValueError(f'unsupported preconfigured {name} value; refusing connection')
    amount=int(match.group(1));unit=(match.group(2) or 'ms').lower()
    scale={'ms':1,'s':1000,'min':60000,'h':3600000,'d':86400000}[unit]
    existing_ms=amount*scale
    # PostgreSQL zero means unlimited; retain the finite host-side cap.
    return min(existing_ms,cap_ms) if existing_ms>0 else cap_ms

def _split_pg_startup_options(options):
    """Match PostgreSQL 18 pg_split_opts: ASCII whitespace and backslash escapes."""
    whitespace=' \t\n\v\f\r';tokens=[];index=0
    while index<len(options):
        while index<len(options) and options[index] in whitespace:index+=1
        if index==len(options):break
        token=[];escaped=False
        while index<len(options):
            char=options[index]
            if char in whitespace and not escaped:break
            if not escaped and char=='\\':escaped=True
            else:
                token.append(char);escaped=False
            index+=1
        tokens.append(''.join(token))
    return tokens

def conninfo(statement_ms=30000,lock_ms=15000,with_limits=False):
    settings=conninfo_to_dict(args.dsn)
    connect=settings.get('connect_timeout')
    if connect is None:connect_seconds=5
    elif re.fullmatch(r'[0-9]+',connect.strip()) is None:
        raise ValueError('unsupported preconfigured connect_timeout; refusing connection')
    else:connect_seconds=min(int(connect),5) if int(connect)>0 else 5
    settings['connect_timeout']=str(connect_seconds)

    options=settings.get('options','')
    caps={'statement_timeout':statement_ms,'lock_timeout':lock_ms}
    index=0
    tokens=_split_pg_startup_options(options)
    while index<len(tokens):
        token=tokens[index]
        if token=='-c':
            if index+1>=len(tokens):raise ValueError('unsupported incomplete PostgreSQL -c option; refusing connection')
            assignment=tokens[index+1]
            if '=' not in assignment:
                if assignment.strip().lower() in caps:raise ValueError(f'unsupported preconfigured {assignment}; refusing connection')
                index+=2;continue
            key,value=assignment.split('=',1);normalized=key.strip().lower()
            if normalized in caps:
                timeout_ms=_bounded_timeout_ms(value,caps[normalized],normalized)
                caps[normalized]=min(caps[normalized],timeout_ms)
            index+=2;continue
        if token[:2].lower()=='-c' and len(token)>2:
            assignment=token[2:]
            if '=' not in assignment:
                if assignment.strip().lower() in caps:raise ValueError(f'unsupported attached {assignment}; refusing connection')
            else:
                key,value=assignment.split('=',1);normalized=key.strip().lower()
                if normalized in caps:
                    timeout_ms=_bounded_timeout_ms(value,caps[normalized],normalized)
                    caps[normalized]=min(caps[normalized],timeout_ms)
            index+=1;continue
        # Do not silently retain an alternate or malformed spelling that may
        # override one of the two host-bounded PostgreSQL timeouts.
        if re.search(r'(?:^|[-=])(?:statement_timeout|lock_timeout)(?:=|$)',token,re.IGNORECASE):
            raise ValueError('unsupported PostgreSQL timeout option spelling; refusing connection')
        index+=1
    # Keep the original option bytes intact: PostgreSQL does not use shell
    # quoting here. Appended -c entries safely take precedence over originals.
    suffix=f'-c statement_timeout={caps["statement_timeout"]}ms -c lock_timeout={caps["lock_timeout"]}ms'
    trailing_backslashes=len(options)-len(options.rstrip('\\'))
    if trailing_backslashes%2:raise ValueError('unsupported dangling PostgreSQL options escape; refusing connection')
    separator=' ' if options else ''
    settings['options']=options+separator+suffix
    result=make_conninfo(**settings)
    return (result,caps) if with_limits else result
def cleanup_timeout(maximum=5):
    return max(0,min(maximum,native_deadline-time.monotonic()-2))
async def bounded_async_wait(awaitables,maximum=5):
    budget=cleanup_timeout(maximum)
    if budget<=0: raise TimeoutError('no host cleanup budget remains')
    done,pending=await asyncio.wait(awaitables,timeout=budget)
    if pending: raise TimeoutError('async cleanup did not finish within the remaining host budget')
    return done
async def bounded_async_close(connection):
    task=asyncio.create_task(connection.close())
    try:
        await bounded_async_wait({task})
    except BaseException:
        task.cancel()
        raise
    task.result()
root=Path(__file__).resolve().parent
source=json.loads((root/'table-operation-model.json').read_text())
payload=source['modules'][0]['extensions']['umf.tables']
source['modules'][0]['elements'][1]['nullability']='absent-allowed'
for command in ('select','insert','update'):
    payload['tables'][0]['policy']['batch'][command]={'kind':'allow'}
for command in payload['commands']:
    if command['id']=='read_page':command['body']['maxRows']=1
    if command['id']=='read_many':command['body']['maxRows']=2
bounded=copy.deepcopy(next(c for c in payload['commands'] if c['id']=='remove'))
bounded.update(id='remove_all_bounded',parameters=[])
bounded['body'].update(where={'op':'all'},maxRows=1)
payload['commands'].append(bounded)
for name,predicate in [
    ('null_safe',{'op':'eq','field':{'module':'demo','element':'owner'},'value':{'kind':'literal','value':None}}),
    ('null_in',{'op':'in','field':{'module':'demo','element':'owner'},'values':[{'kind':'literal','value':None}]}),
    ('not_null_in',{'op':'not','arg':{'op':'in','field':{'module':'demo','element':'owner'},'values':[{'kind':'literal','value':None}]}}),
]:
    read=copy.deepcopy(payload['commands'][0]);read.update(id=name,parameters=[]);read['body'].update(where=predicate,maxRows=100);payload['commands'].append(read)
prefix_read=copy.deepcopy(payload['commands'][0]);prefix_read.update(id='literal_prefix',parameters=[{'id':'key','field':{'module':'demo','element':'id'}}]);prefix_read['body'].update(where={'op':'startsWith','field':{'module':'demo','element':'id'},'value':{'kind':'parameter','parameter':'key'}},maxRows=100)
payload['commands'].append(prefix_read)
prefix='p4_'+uuid.uuid4().hex[:12]+'_'
cfg={'module':'demo','schema':prefix+'schema','ownerRole':prefix+'owner','personRole':prefix+'person','batchRole':prefix+'batch','groupRoles':{}}
alice,bob=prefix+'alice',prefix+'bob'
roles=[cfg['ownerRole'],cfg['personRole'],cfg['batchRole'],alice,bob]
compiled=compile_table_deployment(text=json.dumps(source,separators=(',',':')),configuration=cfg)
assert compiled['status']=='compiled',compiled
bundle=compiled['bundle']
planned=compile_table_migration(migration_id='001',predecessor=None,previous_bundle=None,next_bundle=bundle)
assert planned['status']=='compiled',planned
plan=planned['plan']
binding=json.loads((root/'table-lifecycle-read-template.json').read_text())['binding']
binding.update(schema=cfg['schema'],modelJson=bundle['source']['utf8'],modelSha256=bundle['sourceDigest'])
binding['tables'][0]['columns'][1]['nullable']=True
binding['nativeLifecycle'].update(configurationDigest=bundle['configurationDigest'],deploymentDigest=bundle['bundleDigest'])
observations=[]
program_hashes={}
leases={"acquired":0,"released":0}
server_details={'versionInteger':None,'version':None}
async_lease_details=None
failed_assertion=None
cleanup_status='pending'
cleanup_failure_type=None
session_policy=SessionPolicy(people={"alice":alice,"bob":bob},batches={"scheduled":cfg["batchRole"]},schema=cfg["schema"])


def write_partial_report(status='running',failure_type=None):
    args.report.parent.mkdir(parents=True,exist_ok=True)
    report={'status':status,'scope':'candidate SQL native behavior under source-installed PG18 RLS; not generated/runtime operation admission',
            'postgresql':server_details['versionInteger'],'postgresqlVersion':server_details['version'],
            'weft':importlib.metadata.version('umf-weft'),'sourceDigest':bundle['sourceDigest'],'bundleDigest':bundle['bundleDigest'],
            'observations':observations,'perCallLeases':leases,'asyncLeases':async_lease_details,
            'programDigests':program_hashes,'cleanupStatus':cleanup_status,
            'cleanupFailureType':cleanup_failure_type,'nativeExecutionQualified':False}
    if failed_assertion is not None:report['failedAssertion']=failed_assertion
    if failure_type is not None:report['failureType']=failure_type
    temporary=args.report.with_suffix(args.report.suffix+'.tmp')
    temporary.write_text(json.dumps(report,indent=2)+'\n')
    temporary.replace(args.report)

def record_unhandled(exc_type,exc,tb):
    try:write_partial_report('failed',exc_type.__name__)
    except BaseException:pass
    sys.__excepthook__(exc_type,exc,tb)

sys.excepthook=record_unhandled

def set_cleanup_limits(connection,startup_limits):
    remaining_ms=int(max(0,native_deadline-time.monotonic()-2)*1000)
    if remaining_ms<=0:raise TimeoutError('no host cleanup budget remains for database teardown')
    statement_ms=min(startup_limits['statement_timeout'],remaining_ms)
    lock_ms=min(startup_limits['lock_timeout'],statement_ms,remaining_ms)
    connection.execute("SELECT pg_catalog.set_config('statement_timeout',%s,false),pg_catalog.set_config('lock_timeout',%s,false)",(f'{statement_ms}ms',f'{lock_ms}ms'))

def check(name,value):
    global failed_assertion
    if not value:
        failed_assertion=name
        write_partial_report('failed','AssertionError')
        raise AssertionError(name)
    observations.append(name)
    write_partial_report()


def scalar(value):
    if value is None:return None
    key=next(iter(value))
    return value[key]


def program(name,inputs=None,after=None,items=None):
    call={'kind':'single','inputs':inputs or {}}
    if after is not None:call['after']=after
    if items is not None:call={'kind':'batch','items':items}
    request={'version':'weft.table-operation/0.1','target':'postgresql18','module':'demo','operation':name,'binding':binding,'call':call}
    declaration=next(c for c in payload['commands'] if c['id']==name)
    if declaration['body']['kind']!='read':request['idempotencyKey']='oracle-'+uuid.uuid4().hex
    artifact=json.loads(weft.compile_table_operation_json(json.dumps(request,separators=(',',':'))))
    assert artifact['status']=='compiled',artifact
    verified=json.loads(weft.check_table_operation_json(json.dumps(artifact)))
    assert verified['status']=='checked',verified
    assert artifact['obligations']['nativeExecutionQualified'] is False
    assert artifact['result']['countScope']=='direct-target-rows'
    if declaration['body']['kind']!='read':assert artifact['result']['nativeBoundGuard']['profile']=='pg18-effect-count/0.1'
    program_hashes[name]=artifact['structuralDigest']
    return artifact,call


class BoundExceeded(Exception):pass


def arguments(artifact,row,call,index=0):
    sources={entry['id']:entry['source'] for entry in artifact['parameterSources']}
    values=[]
    for slot in artifact['dbapiParameterOrder']:
        source=sources[slot]
        if source['kind']=='input':value=artifact['nativeInputs'][index][source['parameter']]
        elif source['kind']=='after':value=artifact['nativeAfter'][source['index']]
        else:value=source['nativeValue']
        values.append(value)
    return tuple(values)


def execute(connection,role,name,inputs=None,after=None,items=None,transaction_owner="adapter"):
    artifact,call=program(name,inputs,after,items)
    rows=[]
    identity=SessionIdentity("batch","scheduled") if role==cfg["batchRole"] else SessionIdentity("person","alice" if role==alice else "bob")
    @contextmanager
    def factory(requested):
        assert requested==identity
        leases["acquired"]+=1
        try:yield connection
        finally:leases["released"]+=1
    session=SyncSession(factory,session_policy,transaction_owner=transaction_owner)
    with session.open(identity,read_only=artifact['declaration']['body']['kind']=='read'):
        identity=connection.execute('SELECT current_user,rolsuper,rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=current_user').fetchone()
        assert identity==(role,False,False)
        for index,row in enumerate(call.get('items',[call.get('inputs',{})])):
            # Empty tuples still apply DBAPI percent escaping; no string formatting.
            returned=connection.execute(artifact['dbapiSql'],arguments(artifact,row,call,index)).fetchall()
            result=artifact['result']
            if result['kind']=='mutation' and len(returned)>result['maxRows']:
                raise BoundExceeded()
            if result['kind']=='many' and len(returned)>result['maxRows']:
                raise BoundExceeded()
            if result['kind']=='one' and len(returned)!=1:raise BoundExceeded()
            if result['kind']=='optional' and len(returned)>1:raise BoundExceeded()
            rows.extend(returned)
        assert connection.execute('SELECT current_user').fetchone()==(role,)
    return rows


def owner_input(role):return {'owner':{'string':role}}


def create_input(key,role):return {'id':{'string':key},'owner':None if role is None else {'string':role}}


def refuses(connection,role,name,inputs=None,items=None,error=psycopg.errors.InsufficientPrivilege):
    try:execute(connection,role,name,inputs,items=items)
    except error:pass
    else:raise AssertionError('expected native refusal: '+name)


async def async_oracle(admin):
    global async_lease_details
    connection=await psycopg.AsyncConnection.connect(conninfo(),autocommit=True)
    counters={'acquired':0,'released':0}
    async_lease_details=counters
    task=None
    async def invoke(role,name,inputs,transaction_owner='adapter'):
        artifact,call=program(name,inputs)
        identity=SessionIdentity('batch','scheduled') if role==cfg['batchRole'] else SessionIdentity('person','alice')
        @asynccontextmanager
        async def factory(requested):
            assert requested==identity
            counters['acquired']+=1
            try:yield connection
            finally:counters['released']+=1
        session=AsyncSession(factory,session_policy,transaction_owner=transaction_owner)
        async with session.open(identity,read_only=artifact['declaration']['body']['kind']=='read'):
            current=await connection.execute('SELECT current_user,rolsuper,rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=current_user')
            assert await current.fetchone()==(role,False,False)
            rows=await connection.execute(artifact['dbapiSql'],arguments(artifact,call['inputs'],call))
            return await rows.fetchall()
    try:
        check('async-person-read-through-session-and-compiler',await invoke(alice,'read_count',owner_input(alice))==[('3',)])
        check('async-person-write-through-session-and-compiler',await invoke(alice,'create',create_input('async_owned',alice))==[('async_owned',alice,1)])
        check('async-delete-count-marker',await invoke(alice,'remove',{'id':{'string':'async_owned'}})==[(1,)])
        async with connection.transaction(force_rollback=True):
            cursor=await connection.execute('SELECT current_user');before=await cursor.fetchone()
            pending=await invoke(alice,'create',create_input('async_caller',alice),transaction_owner='caller')
            check('async-caller-savepoint-retains-outer-transaction',int(connection.info.transaction_status)==2 and pending==[('async_caller',alice,1)])
            cursor=await connection.execute('SELECT current_user');check('async-caller-success-restores-role',await cursor.fetchone()==before)
            try:await invoke(alice,'remove_all_bounded',{},transaction_owner='caller')
            except psycopg.errors.DivisionByZero:pass
            else:raise AssertionError('async native bound guard did not fire')
            cursor=await connection.execute('SELECT current_user');check('async-bound-failure-preserves-healthy-caller',int(connection.info.transaction_status)==2 and await cursor.fetchone()==before)
        check('async-caller-rollback-discards-pending-data',await invoke(alice,'read_count',owner_input(alice))==[('3',)])
        with admin.transaction():
            admin.execute(sql.SQL('SELECT id FROM {} WHERE id=%s FOR UPDATE').format(sql.Identifier(cfg['schema'],'notes')),('a1',))
            task=asyncio.create_task(invoke(cfg['batchRole'],'rename',create_input('a1',bob)))
            observed=False
            with psycopg.connect(conninfo(1000,1000),autocommit=True) as monitor:
                deadline=asyncio.get_running_loop().time()+5
                while asyncio.get_running_loop().time()<deadline:
                    state=monitor.execute('SELECT wait_event_type FROM pg_catalog.pg_stat_activity WHERE pid=%s',(connection.info.backend_pid,)).fetchone()
                    if state==('Lock',):observed=True;break
                    if task.done():break
                    await asyncio.sleep(.01)
            check('async-cancellation-observes-real-native-lock-wait',observed)
            task.cancel();task.cancel()
            done=await bounded_async_wait({task})
            if task not in done:raise AssertionError('cancelled write did not finish within host cleanup budget')
            try:task.result()
            except asyncio.CancelledError:pass
            else:raise AssertionError('cancelled write returned a result')
            check('repeated-async-cancellation-propagates-after-cleanup',task.cancelled() and counters['acquired']==counters['released'])
        discarded=connection.closed
        if discarded:connection=await psycopg.AsyncConnection.connect(conninfo(),autocommit=True)
        cursor=await connection.execute("SELECT current_user,pg_catalog.current_setting('role')")
        restored=await cursor.fetchone()
        check('async-cancelled-lease-is-restored-or-discarded',int(connection.info.transaction_status)==0 and restored[0] not in roles and restored[1]=='none')
        check('cancelled-batch-role-write-cannot-transfer-owner',await invoke(alice,'read_count',owner_input(alice))==[('3',)])
        check('all-async-lease-exits-complete',counters['acquired']==counters['released'])
        counters['cancelledConnectionDisposition']='discarded' if discarded else 'restored-and-reused'
        return counters
    finally:
        cleanup_failed=False
        if task is not None and not task.done():
            task.cancel()
            try:await bounded_async_wait({task})
            except TimeoutError:cleanup_failed=True
            if task.done():
                try:task.result()
                except BaseException:pass
        if cleanup_failed:
            try:await bounded_async_close(connection)
            except BaseException:pass
            raise TimeoutError('native operation task did not stop within the remaining host cleanup budget')
        if task is None or task.done():
            await bounded_async_close(connection)


admin_conninfo,admin_startup_limits=conninfo(with_limits=True)
with psycopg.connect(admin_conninfo,autocommit=True) as admin:
    server_details['versionInteger']=admin.info.server_version
    server_details['version']=admin.execute('SELECT version()').fetchone()[0]
    created=[]
    schema_created=False
    try:
        for role in roles:
            admin.execute(sql.SQL('CREATE ROLE {} NOLOGIN NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE NOREPLICATION').format(sql.Identifier(role)));created.append(role)
        for role in (alice,bob):admin.execute(sql.SQL('GRANT {} TO {} WITH INHERIT TRUE,SET FALSE').format(sql.Identifier(cfg['personRole']),sql.Identifier(role)))
        admin.execute(sql.SQL('CREATE SCHEMA {} AUTHORIZATION {}').format(sql.Identifier(cfg['schema']),sql.Identifier(cfg['ownerRole'])));schema_created=True
        with admin.transaction():
            admin.execute(sql.SQL('SET LOCAL ROLE {}').format(sql.Identifier(cfg['ownerRole'])))
            admin.execute('SET LOCAL search_path=pg_catalog')
            installed=apply_table_migration(admin,plan,approved_digest=plan['digest'],expected_head=None,person_roles=(alice,bob))
            check('source-installed-without-data-admission',installed['lifecycleAdmission'] is False)
        for key in ('a1','a2','a3'):
            result=execute(admin,alice,'create',create_input(key,alice));check('person-insert-'+key,result==[(key,alice,1)])
        check('other-person-own-insert',execute(admin,bob,'create',create_input('b1',bob))==[('b1',bob,1)])
        check('batch-insert-unowned-row',execute(admin,cfg['batchRole'],'create',create_input('u1',None))==[('u1',None,1)])
        check('person-count-under-rls',execute(admin,alice,'read_count',owner_input(alice))==[('3',)])
        check('other-person-cannot-read-first-owner',execute(admin,bob,'read_count',owner_input(alice))==[('0',)])
        check('batch-count-has-separate-policy',execute(admin,cfg['batchRole'],'read_count',owner_input(alice))==[('3',)])
        check('null-safe-equality-matches-unowned',execute(admin,cfg['batchRole'],'null_safe')==[('u1',None)])
        check('membership-null-is-sql-unknown',execute(admin,cfg['batchRole'],'null_in')==[])
        check('not-membership-null-retains-unknown',execute(admin,cfg['batchRole'],'not_null_in')==[])
        first=execute(admin,alice,'read_page',owner_input(alice));check('keyset-first-lookahead',first==[('a1',alice),('a2',alice)])
        second=execute(admin,alice,'read_page',owner_input(alice),after=[{'string':'a1'}]);check('keyset-continuation-original-order',second==[('a2',alice),('a3',alice)])
        refuses(admin,alice,'read_many',owner_input(alice),error=BoundExceeded);check('bounded-many-refuses-truncation',True)
        check('native-update-counts-matched-unchanged-row',execute(admin,alice,'rename',create_input('a1',alice))==[('a1',alice,1)])
        check('other-person-update-is-filtered',execute(admin,bob,'rename',create_input('a1',bob))==[])
        refuses(admin,alice,'rename',create_input('a1',bob));check('update-with-check-blocks-owner-transfer',True)
        refuses(admin,alice,'create',create_input('foreign',bob));check('insert-with-check-blocks-other-owner',True)
        check('upsert-ignore-conflict-has-zero-effects',execute(admin,alice,'put_ignore',create_input('a1',alice))==[])
        check('upsert-update-conflict-has-one-effect',execute(admin,alice,'put_update',create_input('a1',alice))==[('a1',alice,1)])
        refuses(admin,alice,'put_update',create_input('b1',alice));check('upsert-cannot-take-other-person-row',True)
        refuses(admin,cfg['batchRole'],'remove',{'id':{'string':'b1'}});check('batch-delete-is-not-person-delete-grant',True)
        refuses(admin,alice,'remove_all_bounded',error=psycopg.errors.DivisionByZero)
        check('native-count-guard-rolls-back-all-delete-effects',execute(admin,alice,'read_count',owner_input(alice))==[('3',)])
        duplicate=create_input('dup',alice)
        refuses(admin,cfg['batchRole'],'create_batch',items=[duplicate,duplicate],error=psycopg.errors.UniqueViolation)
        check('failed-batch-rolls-back-earlier-items',execute(admin,alice,'read_count',owner_input(alice))==[('3',)])
        refuses(admin,alice,'create_batch',items=[create_input('person_bulk_own',alice),create_input('person_bulk_foreign',bob)])
        check('person-bulk-shape-cannot-select-batch-identity',True)
        check('mixed-owner-person-bulk-rolls-back-earlier-items',execute(admin,alice,'read_count',owner_input(alice))==[('3',)])
        inserted=execute(admin,cfg['batchRole'],'create_batch',items=[create_input('c1',alice),create_input('c2',bob)])
        check('batch-preserves-ordered-item-results',inserted==[('c1',alice,1),('c2',bob,1)])
        check('batch-created-person-row-obeys-rls',execute(admin,bob,'read_count',owner_input(alice))==[('0',)])
        check('person-delete-returning-empty-still-proves-count',execute(admin,alice,'remove',{'id':{'string':'c1'}})==[(1,)])
        with admin.transaction(force_rollback=True):
            before_role=admin.execute('SELECT current_user').fetchone()
            pending=execute(admin,alice,'create',create_input('caller_pending',alice),transaction_owner="caller")
            check('caller-savepoint-does-not-commit-outer-transaction',int(admin.info.transaction_status)==2 and pending==[('caller_pending',alice,1)])
            check('caller-success-restores-prior-role',admin.execute('SELECT current_user').fetchone()==before_role)
            try:execute(admin,alice,'remove_all_bounded',transaction_owner="caller")
            except psycopg.errors.DivisionByZero:pass
            else:raise AssertionError('caller bound guard did not fire')
            check('bound-failure-keeps-caller-transaction-healthy',int(admin.info.transaction_status)==2 and admin.execute('SELECT current_user').fetchone()==before_role)
        check('caller-rollback-discards-pending-data',execute(admin,alice,'read_count',owner_input(alice))==[('3',)])
        async_leases=asyncio.run(async_oracle(admin))
        quoted="q%_'\\;--雪"
        check('quoted-unicode-input-remains-data',execute(admin,alice,'create',create_input(quoted,alice))==[(quoted,alice,1)])
        execute(admin,alice,'create',create_input('qZZ',alice))
        check('prefix-percent-underscore-are-literal',execute(admin,alice,'literal_prefix',{'key':{'string':'q%_'}})==[(quoted,alice)])
        with admin.transaction():
            admin.execute(sql.SQL('SET LOCAL ROLE {}').format(sql.Identifier(cfg['ownerRole'])))
            admin.execute('SET LOCAL search_path=pg_catalog')
            history=inspect_table_history(admin,bundle,person_roles=(alice,bob))
            check('data-effects-do-not-rewrite-source-generation',history['headDigest']==installed['entryDigest'])
        check('every-per-call-lease-released-including-refusals',leases['acquired']==leases['released'] and leases['acquired']>30)
        report={'status':'passed','scope':'candidate SQL native behavior under source-installed PG18 RLS; not generated/runtime operation admission',
                'postgresql':server_details['versionInteger'],'postgresqlVersion':server_details['version'],'weft':importlib.metadata.version('umf-weft'),
                'weftModule':Path(weft.__file__).name,'sourceDigest':bundle['sourceDigest'],'bundleDigest':bundle['bundleDigest'],
                'observations':observations,'perCallLeases':leases,'asyncLeases':async_leases,'sessionAdapter':'installed public SyncSession/AsyncSession and Psycopg3; adapter-owned/caller-savepoint transactions plus repeated cancellation','programDigests':program_hashes,'nativeExecutionQualified':False,
                'unqualified':['production host executor','durable data replay','authenticated opaque cursors','generated table methods','Psycopg2/SQLAlchemy operation composition and real-pool/fork parity','managed Lakebase','release']}
    finally:
        cleanup_status='complete'
        active_error=sys.exc_info()[0] is not None
        cleanup_error=None
        if schema_created:
            try:
                set_cleanup_limits(admin,admin_startup_limits)
                admin.execute(sql.SQL('DROP SCHEMA {} CASCADE').format(sql.Identifier(cfg['schema'])))
            except BaseException as exc:
                cleanup_status='failed';cleanup_failure_type=type(exc).__name__;cleanup_error=exc
        for role in reversed(created):
            try:
                set_cleanup_limits(admin,admin_startup_limits)
                admin.execute(sql.SQL('DROP ROLE {}').format(sql.Identifier(role)))
            except BaseException as exc:
                cleanup_status='failed'
                if cleanup_failure_type is None:cleanup_failure_type=type(exc).__name__
                if cleanup_error is None:cleanup_error=exc
        if cleanup_error is not None and not active_error:raise cleanup_error
    report['cleanupStatus']=cleanup_status
    report['cleanupFailureType']=cleanup_failure_type
args.report.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'nativeAssertions':len(observations),'weft':report['weft'],'nativeExecutionQualified':False}))
