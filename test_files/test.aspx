This is a test file.
Files with .config and .aspx have the same content.
2x9 = 18 ok connections, 2x5 = 10 errors.

These 9 should be ok:
<add name="nets" connectionString="Data Source=db2; User Id=dvrpc;" providerName="System.OracleClient"/>
<add name="nets" connectionString="Data Source=db2; User Id=dvrpc;" Providername="System.OracleClient"/>
<add name="nets" connectionString="Data Source=db2; User Id=dvrpc;" ProviderName="System.OracleClient"/>
<add name="nets" connectionString="Data Source=db2; User Id=dvrpc;" providername="System.OracleClient"/>
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

These 5 should err:
<add name="nets" connectionString="" providerName="System.OracleClient"/>
<add name="nets" connectionString="User Id=dvrpc;" providerName="System.OracleClient"/>
<add name="nets" connectionString="Data Source=db2;" providerName="System.OracleClient"/>
<add name="nets" connectionString="Data Source=db2; User Id=dvrpc;" providerInvalid="System.OracleClient"/>
<add 
	name="nets" 
	connectionString="" 
	providerName="System.OracleClient"/>

These should be ignored:	
<connectionStrings>
</connectionStrings>
<this line starts with an angle bracket and contains connectionString>
<name="nets" connectionString="Data Source=db2; User Id=dvrpc;" providerName="System.OracleClient"/>
