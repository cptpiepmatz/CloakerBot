import Stats from "../../enums/Stats";
import BaseUserDBData from "./base/BaseUserDBData";

/**
 * Interface for the database type for stats for users.
 */
export default interface StatsUsersDBData extends BaseUserDBData {
  [Stats.TIMES_COMMANDED_TAKEDOWN]?: {
    [key: string]: number; // The type of key is Snowflake but TS doesn't let me
  };
  [Stats.TIMES_DENIED_COMMAND_TAKEDOWN]?: {
    [key: string]: number;
  };
  [Stats.TIMES_INCORRECT_COMMAND]?: {
    [key: string]: number;
  };
  [Stats.TIMES_TAKEN_DOWN]?: {
    [key: string]: number;
  };
}
