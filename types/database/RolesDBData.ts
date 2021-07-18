import {Snowflake} from "discord.js";

/**
 * Interface for the database type for roles.
 */
export default interface RolesDBData {
  GuildID: Snowflake;
  GuildName: string;
  takedownRole: Snowflake;
}
