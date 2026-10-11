"""Killable host-only OTLP HTTP JSON transport; no raw exception/output channel."""
import http.client,json,sys,urllib.parse

def unique(pairs):
 result={}
 for key,value in pairs:
  if key in result:raise ValueError()
  result[key]=value
 return result

def main():
 try:
  if len(sys.argv)!=2:return 1
  url=urllib.parse.urlsplit(sys.argv[1]);data=sys.stdin.buffer.read(8193)
  if len(data)>8192 or url.scheme not in ('http','https') or not url.hostname or url.username is not None or url.password is not None or url.query or url.fragment or url.path not in ('/v1/logs','/v1/traces'):return 1
  connection=(http.client.HTTPSConnection if url.scheme=='https' else http.client.HTTPConnection)(url.hostname,url.port,timeout=10)
  try:
   connection.request('POST',url.path,data,{'Content-Type':'application/json'})
   response=connection.getresponse();body=response.read(4097)
   if response.status!=200 or len(body)>4096 or response.getheader('Content-Type','').split(';')[0].strip().lower()!='application/json':return 1
   value=json.loads(body,object_pairs_hook=unique)
   return 0 if type(value)==dict and value=={} else 1
  finally:connection.close()
 except BaseException:return 1
if __name__=='__main__':sys.exit(main())
