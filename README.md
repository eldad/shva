# shva
Learning experiment with Axum.

## Why `Shva`?

`Shva` is the hebrew pronounciation of the name `שבא`, Sheba, as in "Queen of Sheba", related to "Aksum" or "Axum".
The Hebrew word "שווא" (also pronounced `Shva`) is a collection of grammatical phonemena, which is spelled - when leaving out the diacritics - the same as the word "שווא" (pronounced approx. `Shav`), meaning "fruitless, in vain".
Though not entirely fruitless, since learning in itself is never a wasted effort!

## Metrics

Prometheus Metrics are available in the `/metrics` endpoint.
Since the `metrics-exporter-prometheus` crate currently does not provider any callback mechanism, I opted out of using the `http-listener`.
By rendering inside axum, one can use the tower `Extension` facility to track the state directly before a scrape call.

## Overload protection

The amount of concurrent requests can be limited. Monitoring endpoints are exempt (currently only `/metrics`).
When the concurrency limit is exceeded, 429 responses are sent to the client (via load shedding middleware).

## Features

- [x] Prometheus metrics
- [x] Concurrency control
- [x] Load shedding
- [x] Compression/Decompression
- [ ] Rate limiting
- [ ] Request timeout
- [x] Basic database access (postgresql)
- [x] Logging and tracing, export to Jaeger

## Performance measurements

Measured with [hey](https://github.com/rakyll/hey) on my personal machine (AMD Ryzen 5 5600H).
Note that the measurements are not very clean: jaeger and postgres were running on the machine, along with other programs such as my browser.

### Simple response

Endpoint: `/`

```
$  ./hey -c 100 -n 1000000 http://localhost:8042/

Summary:
  Total:	11.1287 secs
  Slowest:	0.0707 secs
  Fastest:	0.0001 secs
  Average:	0.0011 secs
  Requests/sec:	89857.5082

  Total data:	2000000 bytes
  Size/request:	2 bytes

Response time histogram:
  0.000 [1]	|
  0.007 [999811]	|■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.014 [122]	|
  0.021 [10]	|
  0.028 [0]	|
  0.035 [53]	|
  0.042 [2]	|
  0.050 [0]	|
  0.057 [0]	|
  0.064 [0]	|
  0.071 [1]	|


Latency distribution:
  10% in 0.0003 secs
  25% in 0.0006 secs
  50% in 0.0009 secs
  75% in 0.0015 secs
  90% in 0.0022 secs
  95% in 0.0027 secs
  99% in 0.0035 secs

Details (average, fastest, slowest):
  DNS+dialup:	0.0000 secs, 0.0000 secs, 0.0018 secs
  DNS-lookup:	0.0000 secs, 0.0000 secs, 0.0019 secs
  req write:	0.0000 secs, 0.0000 secs, 0.0078 secs
  resp wait:	0.0009 secs, 0.0000 secs, 0.0703 secs
  resp read:	0.0001 secs, 0.0000 secs, 0.0143 secs

Status code distribution:
  [200]	1000000 responses
```

### Random error

Endpoint `/random_error`

```
./hey -c 100 -n 1000000 http://localhost:8042/random_error

Summary:
  Total:	44.3266 secs
  Slowest:	0.1140 secs
  Fastest:	0.0001 secs
  Average:	0.0044 secs
  Requests/sec:	22559.8081

  Total data:	11383476 bytes
  Size/request:	11 bytes

Response time histogram:
  0.000 [1]	|
  0.011 [917736]	|■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.023 [59999]	|■■■
  0.034 [21413]	|■
  0.046 [622]	|
  0.057 [154]	|
  0.068 [52]	|
  0.080 [15]	|
  0.091 [3]	|
  0.103 [2]	|
  0.114 [3]	|


Latency distribution:
  10% in 0.0011 secs
  25% in 0.0017 secs
  50% in 0.0026 secs
  75% in 0.0044 secs
  90% in 0.0103 secs
  95% in 0.0168 secs
  99% in 0.0257 secs

Details (average, fastest, slowest):
  DNS+dialup:	0.0000 secs, 0.0000 secs, 0.0035 secs
  DNS-lookup:	0.0000 secs, 0.0000 secs, 0.0022 secs
  req write:	0.0000 secs, 0.0000 secs, 0.0157 secs
  resp wait:	0.0043 secs, 0.0001 secs, 0.1140 secs
  resp read:	0.0001 secs, 0.0000 secs, 0.0177 secs

Status code distribution:
  [417]	198856 responses
  [418]	199434 responses
  [421]	199655 responses
  [500]	198997 responses
  [501]	203058 responses
```

### Database Ping

Note that postgres AND jaeger were running at the same time.

```

Summary:
  Total:	99.0406 secs
  Slowest:	0.1514 secs
  Fastest:	0.0004 secs
  Average:	0.0099 secs
  Requests/sec:	10096.8697


Response time histogram:
  0.000 [1]	|
  0.016 [975075]	|■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.031 [22255]	|■
  0.046 [1206]	|
  0.061 [97]	|
  0.076 [106]	|
  0.091 [203]	|
  0.106 [204]	|
  0.121 [424]	|
  0.136 [397]	|
  0.151 [32]	|


Latency distribution:
  10% in 0.0071 secs
  25% in 0.0081 secs
  50% in 0.0094 secs
  75% in 0.0109 secs
  90% in 0.0125 secs
  95% in 0.0137 secs
  99% in 0.0195 secs

Details (average, fastest, slowest):
  DNS+dialup:	0.0000 secs, 0.0000 secs, 0.0038 secs
  DNS-lookup:	0.0000 secs, 0.0000 secs, 0.0020 secs
  req write:	0.0000 secs, 0.0000 secs, 0.0086 secs
  resp wait:	0.0098 secs, 0.0004 secs, 0.1512 secs
  resp read:	0.0000 secs, 0.0000 secs, 0.0071 secs

Status code distribution:
  [200]	1000000 responses
```
