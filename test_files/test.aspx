This is a test file.
All files with .config, .asp, and .aspx have the same contents
so 6x3=18 connections and 7x3=21 errors

These 6 should be ok:
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

These 7 should err:
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
