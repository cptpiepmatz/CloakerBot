import BaseUserDBData from "./base/BaseUserDBData";

/**
 * Interface for the database type for stats for users.
 */
export default interface StatsUsersDBData extends BaseUserDBData {
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
