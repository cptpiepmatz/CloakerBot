import {Snowflake} from "discord.js";

type Digit = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9;

/**
 * Base interface for the database types with users.
 */
export default interface BaseUserDBData {
  UserID: Snowflake;
  UserName: string;
  UserDiscriminator: `${Digit}${Digit}${Digit}${Digit}`;
}
