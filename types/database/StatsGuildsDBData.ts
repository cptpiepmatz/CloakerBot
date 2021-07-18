import BaseGuildDBData from "./base/BaseGuildDBData";

/**
 * Interface for the database type for stats for guilds.
 */
export default interface StatsGuildsDBData extends BaseGuildDBData {
  takedownSuccess?: number;
  randomTakedowns?: number;
  botsTakenDown?: number;
}
