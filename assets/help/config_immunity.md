Configure roles to be immune to takedowns.


It is possible to configure roles that are immune to takedowns.
The `general` option is for configuring roles that will be protected by any kind 
of takedowns executed by the {}.
The bot will still try to take that down user but will fail.

The `random` option is for configuring roles that will only be protected from 
random takedowns.
When the random selection lands on user with such a role, the random takedown 
will fail.

The `*-silent` options allow roles to not only be immune but to never be a 
target for a takedown.
Calls on these users will instantly be rejected.
