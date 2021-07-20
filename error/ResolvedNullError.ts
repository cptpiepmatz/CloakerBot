import {
  ApplicationCommandResolvable,
  ApplicationResolvable,
  Base64Resolvable,
  BitFieldResolvable,
  BufferResolvable,
  CategoryChannelResolvable,
  ChannelResolvable,
  ColorResolvable,
  DateResolvable,
  EmojiIdentifierResolvable,
  EmojiResolvable,
  GuildBanResolvable,
  GuildChannelResolvable,
  GuildMemberResolvable,
  GuildResolvable,
  GuildTemplateResolvable,
  GuildTextChannelResolvable,
  InviteResolvable,
  MessageActionRowComponentResolvable,
  MessageButtonStyleResolvable,
  MessageComponentTypeResolvable,
  MessageReactionResolvable,
  MessageResolvable,
  OverwriteResolvable,
  PermissionOverwriteResolvable,
  PermissionResolvable,
  PresenceResolvable,
  RoleResolvable,
  StageInstanceResolvable,
  SystemChannelFlagsResolvable,
  ThreadChannelResolvable,
  ThreadMemberResolvable,
  UserResolvable
} from "discord.js";

/** Maybe a bit over the top collection of Resolvable types from Discord.js */
type Resolvable =
  ApplicationCommandResolvable |
  ApplicationResolvable |
  Base64Resolvable |
  BitFieldResolvable<any, any> |
  BufferResolvable |
  CategoryChannelResolvable |
  ChannelResolvable |
  ColorResolvable |
  DateResolvable |
  EmojiIdentifierResolvable |
  EmojiResolvable |
  GuildBanResolvable |
  GuildChannelResolvable |
  GuildMemberResolvable |
  GuildResolvable |
  GuildTemplateResolvable |
  GuildTextChannelResolvable |
  InviteResolvable |
  MessageActionRowComponentResolvable |
  MessageButtonStyleResolvable |
  MessageComponentTypeResolvable |
  MessageReactionResolvable |
  MessageResolvable |
  OverwriteResolvable |
  PermissionOverwriteResolvable |
  PermissionResolvable |
  PresenceResolvable |
  RoleResolvable |
  StageInstanceResolvable |
  SystemChannelFlagsResolvable |
  ThreadChannelResolvable |
  ThreadMemberResolvable |
  UserResolvable

/**
 * Error to be used if the resolved value was null but a non null value was
 * needed.
 */
class ResolvedNullError extends Error {
  /**
   * @param toResolve Object that should be resolved
   * @param resolver Object that tried to resolve
   */
  constructor(toResolve: Resolvable, resolver: any) {
    super(
      `Tried to resolve ${toResolve.toString()} with ${resolver} but only got null. `
      + "The null is not an error, it is expected but not wished here.");
  }
}

export default ResolvedNullError;
