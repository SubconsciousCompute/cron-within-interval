[![Build](https://github.com/SubconsciousCompute/cron-with-randomness/actions/workflows/rust.yml/badge.svg)](https://github.com/SubconsciousCompute/cron-with-randomness/actions/workflows/rust.yml)
# cron-with-randomness

Extended cron shorthand to randomly select value from given interval.


In addition to standard expression supported by excellent crate cron, we support
following type of expressions.

- `@daily{H=9-17}` means run once between 9am and 5pm chosen randomly.  
- `@daily{H=9-12,H=15-20}` means run once between 9am and 12pm or between 3pm and 8pm.

Similarly one can pass daily contraints to @weekly.

- `@weekly{D=1-5}` mean  run once per week between day 1 and day 5.  
- `@weekly{D=1-5,H=9-12}` run once per week between day 1 and day 5 and between 9am
   and 12pm.  
- `@weekly{H=9-12}` run once per week at any day chosen randomly and between 9am
   and 12pm.

# Examples

## `@daily`

```
$ cargo run -- @daily{9-17}

 --> 2023-12-18T13:17:00Z
 --> 2023-12-19T09:52:00Z
 --> 2023-12-20T13:12:00Z
 --> 2023-12-21T16:44:00Z
 --> 2023-12-22T09:09:00Z
 --> 2023-12-23T09:37:00Z
 --> 2023-12-24T16:55:00Z
 --> 2023-12-25T12:28:00Z
 --> 2023-12-26T16:36:00Z
 --> 2023-12-27T10:35:00Z
```
 
## `@weekly`

```
$ cargo run -- @weekly{h=0-2}

 --> 2023-12-24T00:16:00Z
 --> 2023-12-31T00:06:00Z
 --> 2024-01-07T00:28:00Z
 --> 2024-01-14T00:24:00Z
 --> 2024-01-21T00:43:00Z
 --> 2024-01-28T00:33:00Z
 --> 2024-02-04T01:20:00Z
 --> 2024-02-11T00:28:00Z
 --> 2024-02-18T00:04:00Z
 --> 2024-02-25T01:22:00Z
 ```


```
$ cargo run -- '@weekly{d=1-4,h=0-2}'
 --> 2023-12-25T01:51:00Z
 --> 2024-01-03T01:02:00Z
 --> 2024-01-09T00:27:00Z
 --> 2024-01-15T01:10:00Z
 --> 2024-01-24T01:10:00Z
 --> 2024-01-30T00:53:00Z
 --> 2024-02-07T00:31:00Z
 --> 2024-02-13T00:18:00Z
 --> 2024-02-19T00:35:00Z
 --> 2024-02-26T01:16:00Z
 ```
