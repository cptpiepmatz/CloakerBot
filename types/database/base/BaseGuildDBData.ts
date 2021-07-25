import {Snowflake} from "discord.js";

/**
 * Base interface for the database types with guilds.
 */
export default interface BaseGuildDBData {
  GuildID: Snowflake;
  GuildName: string;
}
