import {Snowflake} from "discord.js";

/**
 * Interface for MemberRanks.
 * Mostly used for the ranks badge.
 */
export default interface MemberRanks {
  UserID: Snowflake;
  UserName: string;
  victimRankHere: number;
  victimRankTotal: number;
  punisherRankHere: number;
  punisherRankTotal: number;
  timesSmackedHere: number;
  timesSmackedTotal: number;
  timesCalledHere: number;
  timesCalledTotal: number;
}
