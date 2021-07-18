import {Snowflake} from "discord.js";

type Digit = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9;

/**
 * Interface for the database type for stats for users.
 */
export default interface StatsUsersDBData {
  UserID: Snowflake;
  UserDiscriminator: `${Digit}${Digit}${Digit}${Digit}`;
  UserName: string;
  timesCommandedTakedown?: {
    [key: string]: number; // The type of key is Snowflake but TS doesn't let me
  };
  timesTakenDown?: {
    [key: string]: number;
  };
  timesDeniedCommandedTakedown?: {
    [key: string]: number;
  };
}
