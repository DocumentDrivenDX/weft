"""Test-only SELECT projection annotation; leaves query clauses and ordering intact."""
PROJECTION=" to_json(current_version(), map('ignoreNullFields','false')) AS __weft_warehouse, "
def capture(sql):
    positions=[];depth=0;quote=None;i=0
    while i<len(sql):
        ch=sql[i]
        if quote:
            if ch==quote:
                if i+1<len(sql) and sql[i+1]==quote:i+=2;continue
                quote=None
            i+=1;continue
        if ch in "'\"`":quote=ch;i+=1;continue
        if ch=='(':depth+=1
        elif ch==')':depth-=1;assert depth>=0
        elif ch.isalpha() or ch=='_':
            start=i
            while i<len(sql) and (sql[i].isalnum() or sql[i]=='_'):i+=1
            if depth==0 and sql[start:i].upper()=='SELECT':positions.append(i)
            continue
        i+=1
    assert quote is None and depth==0 and len(positions)==1,'Expected one outer SELECT in compiler SQL'
    at=positions[0]
    return sql[:at]+PROJECTION+sql[at:]
