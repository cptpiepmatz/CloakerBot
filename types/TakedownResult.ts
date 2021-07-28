import {GuildMember} from "discord.js";

/** Interface for results of a takedown. */
export default interface TakedownResult {
  target: GuildMember;
  success: boolean;
  message?: string;
}
