Configure the random takedowns.

The command opens a form to enable or disable random takedowns and edit both
intervals. Changes are saved only when you submit; cancelling changes nothing.
Optional command arguments prefill the form. Leave an interval blank or enter
`-1` to restore its default (minimum: 1 hour, maximum: 24 hours).
Intervals must be positive, at most 8760 hours, and the minimum cannot exceed the maximum.


The {CloakerBot} is able to perform randomized takedowns. 
If enabled, the bot will check for a user to take down within a specified time 
interval determined by the `min` and `max` values, which represent the minimum and 
maximum time between checks, respectively. 
These values are in hours and can include decimal points 
(for example, `2` for two hours or `0.25` for 15 minutes).
Even if no takedown occurred, a new moment in the future honoring the set time 
interval will be selected.

Changing any settings related to randomized takedowns will result in the 
selection of a new future time, potentially replacing a scheduled takedown in 
the near future.

Checks for random takedowns will happen every few seconds, therefore the minimum 
time interval between takedowns are these random takedown checks.

For a randomized takedown to happen, there must be a minimum of three users in 
the voice channels of the server.
