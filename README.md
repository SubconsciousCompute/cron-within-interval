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
