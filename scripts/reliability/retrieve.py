"""Read-only bounded run inspection; stdout is one validated JSON page."""
import argparse,json,pathlib,sys
if __package__ in (None,''):sys.path.insert(0,str(pathlib.Path(__file__).resolve().parents[1]))
from reliability.config import SafeParser

def main():
 try:
  parser=SafeParser();parser.add_argument('--run-directory',required=True);parser.add_argument('--cursor');options=parser.parse_args()
  from reliability.diagnostics import retrieve
  page=retrieve(pathlib.Path(options.run_directory),options.cursor)
  print(json.dumps(page,separators=(',',':'),sort_keys=True));return 0
 except Exception:
  print('weft-runner: diagnostic retrieval refused',file=sys.stderr);return 1
if __name__=='__main__':sys.exit(main())
