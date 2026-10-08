"""Test-only host executor for the pinned original entity obligation subset.
Callbacks belong to the host: they provide transaction-affine reads and SQL.
This fixture proves orchestration, not callback trust or production enforcement.
"""
import re

class Refused(Exception):
    pass

def require(condition):
    if not condition:
        raise Refused('WFT-OBLIGATION')

def execute(artifact, snapshot, query):
    try:
        require(artifact['status']=='compiled')
        context=None;complete=None;guards=[]
        for obligation in artifact['obligations']:
            require(obligation['owner']=='host')
            identity=obligation['id'];parameters=obligation['parameters']
            if identity=='truss.candidate.context':
                require(context is None)
                require(parameters['execution']==dict(pinsRecheckedPerExecution=True,sameAffineTransactionForIntegrityAndData=True,separatePagesDoNotImplySnapshotContinuity=True,unknownObligationMeaning='refuse-before-sql'))
                require(parameters['visibility']==dict(absenceRequiresCompleteStateView=True,authorityOwner='host',compoundRequiresCompleteChildView=True,hiddenRowsAreNotAbsent=True))
                require(parameters['requirements']==[
                    'same admitted catalog/layout/model/role view',
                    'exact native transport',
                    'exact numeric domain and finite SUM or runtime error',
                    'integrity checks before predicates/casts; no partial publication',
                    'complete authorized visibility before interpreting missing state or children',
                    'current authority and disclosure rechecked before publication'])
                require(set(parameters)=={'execution','visibility','requirements'})
                context=parameters
            elif identity=='truss.original.complete-read-context':
                require(complete is None)
                require(parameters==dict(beforePublication=True,bindingSha256=artifact['bindingSha256'],completeOwnerVisibility=True,sameTransactionAndAuthorization=True))
                complete=parameters
            elif re.fullmatch(r'truss\.original\.owner-(payload|structural)-[0-9]+',identity):
                require(parameters['beforeQuery'] is True and parameters['expectedViolations']=='0')
                require(parameters['parameters']==artifact['parameters'])
                require(isinstance(parameters['sql'],str) and parameters['sql'])
                guards.append(parameters)
            else:
                raise Refused('unknown obligation')
        require(context is not None and complete is not None and guards)
        def validate(view):
            require(view['bindingSha256']==artifact['bindingSha256'])
            require(view['modelPins']==artifact['modelPins'])
            require(view['targetContext']==artifact['targetContext'])
            require(view['completeVisibility'] is True)
            require(isinstance(view['authority'],str) and view['authority'])
        initial=snapshot();validate(initial)
        for guard in guards:
            require(query(guard['sql'],artifact['parameters'])==[('0',)])
        # Buffer all rows; callbacks may not publish provisional results.
        rows=query(artifact['sql'],artifact['parameters'])
        current=snapshot();validate(current)
        require(current['authority']==initial['authority'])
        return rows
    except Refused:
        raise
    except Exception as error:
        raise Refused('host refused execution/publication') from error
