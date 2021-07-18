import {Snowflake} from "discord.js";

/**
 * Interface for the database type for stats for guilds.
 */
export default interface StatsGuildsDBData {
  GuildID: Snowflake;
  GuildName: string;
  takedownSuccess?: number;
  randomTakedowns?: number;
  botsTakenDown?: number;
}
