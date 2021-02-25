"use strict";

const guildCollection = "stats_guilds";
const userCollection = "stats_users";

/**
 * Function to increase stats
 *
 * @param {String|String[]|incStat.STATS|incStat.STATS[]} statsToInc - stats to increment in the database
 * @param {Guild} guild where the stat increase happened
 * @param {?GuildMember} [commander=null] who commanded the takedown
 * @param {?GuildMember} [target=null] who was taken down down
 * @returns {Promise<void>}
 */
async function incStat(statsToInc, guild, commander = null, target = null) {
  if (!Array.isArray(statsToInc)) statsToInc = Array.of(statsToInc);
  if (!statsToInc.every(stat => Object.values(incStat.STATS).contains(stat))) return;


const STATS = {
  // How often a bot was taken down on a guild
  BOTS_TAKEN_DOWN: "botsTakenDown",

  // How often people commanded a takedown on a guild
  COMMANDED_TAKEDOWNS: "commandedTakedowns",

  // How often commanded takedown was denied on a guild
  DENIED_TAKEDOWNS: "deniedTakedowns",

  // How often a takedown failed on a guild
  FAILED_TAKEDOWNS: "failedTakedowns",

  // How often someone was randomly taken down on a guild
  RANDOM_TAKEDOWNS: "randomTakedowns",

  // How often takedowns failed on a guild
  TAKEDOWNS_FAILED: "takedownsFailed",

  // How often takedowns were successful on a guild
  TAKEDOWNS_SUCCESS: "takedownsSuccess",

  // How often a user commanded a takedown per guild
  TIMES_COMMANDED_TAKEDOWN: "timesCommandedTakedown",

  // How often a user was denied to command a takedown per guild
  TIMES_DENIED_COMMAND_TAKEDOWN: "timesDeniedCommandedTakedown",

  // How often a user entered an incorrect command
  TIMES_INCORRECT_COMMAND: "timesIncorrectCommand",

  // How often a user was taken down per guild
  TIMES_TAKEN_DOWN: "timesTakenDown"
};

const STATS_NEED_COMMANDER = [
  STATS.TIMES_COMMANDED_TAKEDOWN,
  STATS.TIMES_DENIED_COMMAND_TAKEDOWN,
  STATS.TIMES_INCORRECT_COMMAND
];

const STATS_NEED_TARGET = [
  STATS.TIMES_TAKEN_DOWN
];

incStat.STATS = STATS;

module.exports = incStat;