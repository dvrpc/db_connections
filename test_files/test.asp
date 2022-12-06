This is a test file.
It should produce 2 connections and 3 errors.

This should be ok:
"Data Source=db2; User Id=dvrpc; Provider=System.OracleClient"
"Data Source=db2; User Id=dvrpc; providerName=System.OracleClient"

These should be errors:
"Data Source=db2; Provider=System.OracleClient"
"User Id=dvrpc; Provider=System.OracleClient"
"Provider=System.OracleClient"
