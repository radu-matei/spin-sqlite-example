# Read-only SQLite database with Spin


```
$ spin build && spin up

$ curl -X POST http://localhost:3000/query -d "SELECT * FROM Artists LIMIT 5"
[{"ArtistId":1,"Name":"AC/DC"},{"ArtistId":2,"Name":"Accept"},{"ArtistId":3,"Name":"Aerosmith"},{"ArtistId":4,"Name":"Alanis Morissette"},{"ArtistId":5,"Name":"Alice In Chains"}]%
```