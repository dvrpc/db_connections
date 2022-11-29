This is a test file.
It should produce 1 connection and 4 errors.

This should be ok:
"Data Source=db2; User Id=dvrpc; Provider=System.OracleClient"

These should be errors:
"Data Source=db2; User Id=dvrpc; ProviderName=System.OracleClient"
"Data Source=db2; Provider=System.OracleClient"
"User Id=dvrpc; Provider=System.OracleClient"
"Provider=System.OracleClient"


These should just be ignored:
<add name="nets" connectionString="Data Source=db2; User Id=dvrpc;" providerName="System.OracleClient"/>
<add name="nets" connectionString="Data Source=db2; User Id=dvrpc;" provider="System.OracleClient"/>
<add name="nets" connectionString="Data Source=db2; User Id=dvrpc; Password=something;" providerName="System.OracleClient"/>
<add name="nets" connectionString="Data Source=(DESCRIPTION=(ADDRESS=(PROTOCOL=tcp)(HOST=www.dvrpc.org)(PORT=9999))(CONNECT_DATA=(SERVICE_NAME=www.dvrpc.org))); Min Pool Size=0;  User Id=dvrpc;" provider="Oracle.Client"/> 
<add 
	name="nets" 
	connectionString="Data Source=db2; 
	User Id=dvrpc;" 
	providerName="System.OracleClient"/>
<add 
	name="nets" 
	connectionString="Data Source=db2; 
	User Id=dvrpc;" 
	providerName="System.OracleClient"
/>
<add name="nets" connectionString="" providerName="System.OracleClient"/>
<add name="nets" connectionString="User Id=dvrpc;" providerName="System.OracleClient"/>
<add name="nets" connectionString="Data Source=db2;" providerName="System.OracleClient"/>
<add name="nets" connectionString="Data Source=db2; User Id=dvrpc;" providerInvalid="System.OracleClient"/>
<name="nets" connectionString="Data Source=db2; User Id=dvrpc;" providerName="System.OracleClient"/>
<this line starts with an angle bracket and contains connectionString>
<add 
	name="nets" 
	connectionString="" 
	providerName="System.OracleClient"/>
