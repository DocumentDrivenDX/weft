"""Host-only, typed fail-closed configuration. No compiler imports."""
from __future__ import annotations
import argparse, dataclasses, hashlib, json, math, os, pathlib, stat, subprocess, urllib.parse, re

class ConfigurationError(Exception):
    """Safe error category; callers must not echo raw configuration or OS errors."""

@dataclasses.dataclass(frozen=True)
class Config:
    cargo: pathlib.Path
    rustup_home: pathlib.Path
    cargo_home: pathlib.Path
    temp_root: pathlib.Path
    output_root: pathlib.Path
    timeout_seconds: float = 1800
    record_limit: int = 256
    queue_limit: int = 16
    retained_runs: int = 8
    export_timeout_seconds: float = 2
    shutdown_timeout_seconds: float = 3
    endpoint: str | None = None

    def command_environment(self) -> dict[str,str]:
        result = os.environ.copy()
        result.update(RUSTUP_HOME=str(self.rustup_home), CARGO_HOME=str(self.cargo_home), RUSTUP_TOOLCHAIN='1.90.0', RUSTUP_AUTO_INSTALL='0')
        result.update(RUSTC=str(self.cargo.parent/'rustc'),RUSTC_WRAPPER='',RUSTC_WORKSPACE_WRAPPER='')
        result['PATH'] = str(self.cargo.parent) + os.pathsep + result.get('PATH','')
        return result

    def validate_diagnostics_environment(self) -> None:
        if any(key.startswith('OTEL_') for key in os.environ):raise ConfigurationError()

    def validate_repository(self, root:pathlib.Path) -> None:
        try:
            source=root.resolve()
            for path in (self.temp_root,self.output_root):
                operational=path.resolve()
                if operational==source or source in operational.parents:raise ConfigurationError()
        except (OSError,ValueError,UnicodeError):raise ConfigurationError() from None

    def tool_identity(self, root:pathlib.Path) -> dict:
        self.validate_repository(root)
        try:
            env=self.command_environment()
            from .process import run
            def probe(tool,name):
                result=run([str(tool),'--version'],root,env,5,capture_limit=256)
                if result.timed_out or result.exit_code!=0 or result.bytes_read>256:raise ConfigurationError()
                text=result.captured.decode('ascii')
                if not re.fullmatch(name+r' 1\.90\.0 \([0-9a-f]{9} [0-9]{4}-[0-9]{2}-[0-9]{2}\)\n',text):raise ConfigurationError()
                return text.rstrip('\n')
            cargo=probe(self.cargo,'cargo')
            rustc=probe(self.cargo.parent/'rustc','rustc')
            import tomllib
            toolchain=(root/'rust-toolchain.toml').read_bytes()
            if tomllib.loads(toolchain.decode())['toolchain']['channel']!='1.90.0':raise ConfigurationError()
            digest=hashlib.sha256()
            with self.cargo.open('rb') as stream:
                for chunk in iter(lambda:stream.read(65536),b''):digest.update(chunk)
            return {'cargoVersion':cargo,'rustcVersion':rustc,'cargoExecutableSha256':digest.hexdigest(),'rustToolchainSha256':hashlib.sha256(toolchain).hexdigest()}
        except (OSError,ValueError,KeyError,subprocess.SubprocessError) as exc:
            raise ConfigurationError() from None

_FIELDS={f.name for f in dataclasses.fields(Config)}
_REQUIRED={'cargo','rustup_home','cargo_home','temp_root','output_root'}
_PATHS=_REQUIRED
_INTS={'record_limit':(4,256),'queue_limit':(1,256),'retained_runs':(1,32)}
_FLOATS={'timeout_seconds':(1,3600),'export_timeout_seconds':(.05,10),'shutdown_timeout_seconds':(.1,15)}
class SafeParser(argparse.ArgumentParser):
    def error(self,message):raise ConfigurationError()

def _pairs(pairs):
    result={}
    for k,v in pairs:
        if k in result:raise ConfigurationError()
        result[k]=v
    return result

def load_config(argv:list[str], environ:dict[str,str]|None=None) -> Config:
    env=os.environ if environ is None else environ
    parser=SafeParser(add_help=True)
    parser.add_argument('--config')
    for name in sorted(_FIELDS):parser.add_argument('--'+name.replace('_','-'))
    try:
        options=vars(parser.parse_args(argv));values={}
        file=options.pop('config')
        if file is not None:
            handle=os.open(file,os.O_RDONLY | os.O_NONBLOCK | getattr(os,'O_NOFOLLOW',0))
            with os.fdopen(handle,'rb') as stream:
                if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):raise ConfigurationError()
                raw=stream.read(65537)
            if len(raw)>65536:raise ConfigurationError()
            values=json.loads(raw,object_pairs_hook=_pairs)
            if type(values) is not dict or set(values)-_FIELDS:raise ConfigurationError()
        # Unknown WEFT_* names refuse instead of silently ignoring typoed settings.
        allowed={'WEFT_'+name.upper() for name in _FIELDS}
        if any(key.startswith('WEFT_') and key not in allowed for key in env):raise ConfigurationError()
        for name in _FIELDS:
            key='WEFT_'+name.upper()
            if key in env:values[name]=env[key]
            if options[name] is not None:values[name]=options[name]
        if not _REQUIRED <= values.keys():raise ConfigurationError()
        for name,value in list(values.items()):
            if name in _PATHS:
                if not isinstance(value,str) or not value or '\x00' in value or not pathlib.Path(value).is_absolute():raise ConfigurationError()
                values[name]=pathlib.Path(value)
            elif name in _INTS:
                if isinstance(value,str):value=json.loads(value)
                low,high=_INTS[name]
                if type(value) is not int or not low<=value<=high:raise ConfigurationError()
                values[name]=value
            elif name in _FLOATS:
                if isinstance(value,str):value=json.loads(value)
                low,high=_FLOATS[name]
                if type(value) not in (int,float) or not math.isfinite(value) or not low<=value<=high:raise ConfigurationError()
                values[name]=float(value)
            elif name=='endpoint':
                if value is not None:
                    if not isinstance(value,str):raise ConfigurationError()
                    url=urllib.parse.urlsplit(value)
                    if url.scheme not in ('http','https') or not url.hostname or url.username is not None or url.password is not None or url.query or url.fragment or url.path!='/v1/logs':raise ConfigurationError()
                    _=url.port
        config=Config(**values)
        if not config.cargo.is_file() or not os.access(config.cargo,os.X_OK):raise ConfigurationError()
        if any(not getattr(config,name).is_dir() for name in ('rustup_home','cargo_home')):raise ConfigurationError()
        for name in ('temp_root','output_root'):
            path=getattr(config,name)
            if path.exists() and not path.is_dir():raise ConfigurationError()
        return config
    except (OSError,ValueError,TypeError,OverflowError,UnicodeError) as exc:
        raise ConfigurationError() from None
