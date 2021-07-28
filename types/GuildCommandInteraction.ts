import {CommandInteraction, Guild} from "discord.js";

/** A command interaction that has a guild for sure. */
export default interface GuildCommandInteraction extends CommandInteraction {
  guild: Guild
}
