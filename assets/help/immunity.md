Configure roles to be immune to takedowns.

Open `/config` and click Configure beside Immunity to edit all four immunity types
in one form with their current roles selected. Submit to save, or clear a selection
and submit to unset those roles. Cancelling changes nothing.


The {CloakerBot} supports four types of role-based immunity:

- **General** protects against all takedowns. The bot can still start a takedown,
  but it will fail instead of disconnecting the target.
- **Random** protects against random takedowns only. If a protected user is
  selected, the takedown will fail.
- **General silent** prevents takedowns from starting against protected users.
- **Random silent** prevents random takedowns from starting against protected users,
  but does not protect against other kinds of takedowns.
