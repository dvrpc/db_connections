This is a test file.
It should produce 5 connections and 0 errors.

This should be ok:
"Data Source=db2; User Id=dvrpc; Provider=System.OracleClient"
"Data Source=db2; User Id=dvrpc; providerName=System.OracleClient"
"Data Source=db2; Provider=System.OracleClient"
"User Id=dvrpc; Provider=System.OracleClient"
"Provider=Only.A.Provider"

These should be ignored:
"Data Source=;Provider=;User Id=;"
"Data Source=;Provider=;User Id="
