import BaseGuildDBData from "./base/BaseGuildDBData";

/**
 * Interface for the database type for random takedowns.
 */
export default interface RandomTakedownsDBData extends BaseGuildDBData {
  doRandomTakedowns: boolean;
  randomTakedownMin?: number;
  randomTakedownMax?: number;
  takedownDate?: Date | null;
}
