import BaseGuildDBData from "./base/BaseGuildDBData";
import Stats from "../../enums/Stats";

/**
 * Interface for the database type for stats for guilds.
 */
export default interface StatsGuildsDBData extends BaseGuildDBData {
  [Stats.BOTS_TAKEN_DOWN]?: number;
  [Stats.COMMANDED_TAKEDOWNS]?: number;
  [Stats.DENIED_TAKEDOWNS]?: number;
  [Stats.RANDOM_TAKEDOWNS]?: number;
  [Stats.TAKEDOWNS_FAILED]?: number;
  [Stats.TAKEDOWNS_SUCCESS]?: number;
}
