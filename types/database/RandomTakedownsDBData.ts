import {Snowflake} from "discord.js";

/**
 * Interface for the database type for random takedowns.
 */
export default interface RandomTakedownsDBData {
  doRandomTakedowns: boolean;
  GuildID: Snowflake;
  GuildName: String;
  randomTakedownMin?: number;
  randomTakedownMax?: number;
  takedownDate?: Date | null;
}
