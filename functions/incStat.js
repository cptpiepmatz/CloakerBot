"use strict";

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
  const guildCollection = dbo.collection("stats_guilds");
  const userCollection = dbo.collection("stats_users");

  let updateCommander = false;
  let updateTarget = false;

  if (!Array.isArray(statsToInc)) statsToInc = Array.of(statsToInc);
  if (!statsToInc.every(stat => Object.values(incStat.STATS).contains(stat))) return;

  // queries for the database
  let guildQuery = {
    GuildID: guild.id
  }
  let commanderQuery = {};
  let targetQuery = {};

  if (!commander) {
    let intersectNeedsCommander =
      STATS_NEED_COMMANDER.filter(x => statsToInc.includes(x));
    if (intersectNeedsCommander.length !== 0) {
      throw new Error(intersectNeedsCommander.join(", ") + "needs a commander.");
    }
    commanderQuery = {
      UserID: commander.id
    }
    updateCommander = true;
  }

  if (!target) {
    let intersectNeedsTarget =
      STATS_NEED_TARGET.filter(x => statsToInc.includes(x));
    if (intersectNeedsTarget.length !== 0) {
      throw new Error(intersectNeedsTarget.join(", ") + "needs a target.");
    }
    targetQuery = {
      UserID: target.id
    }
    updateTarget = true;
  }

  let guildData = {
    $set: {
      GuildID: guild.id,
      GuildName: guild.name
    },
    $inc: undefined
  }

  let commanderData = {
    $set: {
      UserID: commander.user.id,
      UserName: commander.user.username,
      UserDiscriminator: commander.user.discriminator
    }
  }

  let targetData = {
    $set: {
      UserID: target.user.id,
      UserName: target.user.username,
      UserDiscriminator: target.user.discriminator
    }
  }

  function updateStatGuild(stat) {
    if (statsToInc.includes(stat)) {
      guildData.$inc[stat] = 1;
    }
  }
  updateStatGuild(STATS.BOTS_TAKEN_DOWN);
  updateStatGuild(STATS.COMMANDED_TAKEDOWNS);
  updateStatGuild(STATS.DENIED_TAKEDOWNS);
  updateStatGuild(STATS.FAILED_TAKEDOWNS);
  updateStatGuild(STATS.RANDOM_TAKEDOWNS);
  updateStatGuild(STATS.TAKEDOWNS_FAILED);
  updateStatGuild(STATS.TAKEDOWNS_SUCCESS);

  function updateStatCommander(stat, guild) {
    if (statsToInc.includes(stat)) {
      commanderData.$inc[stat + "." + guild] = 1;
    }
  }
  updateStatCommander(STATS.TIMES_COMMANDED_TAKEDOWN, guild);
  updateStatCommander(STATS.TIMES_DENIED_COMMAND_TAKEDOWN, guild);
  updateStatCommander(STATS.TIMES_INCORRECT_COMMAND, guild);

  function updateStatTarget(stat, guild) {
    if (statsToInc.includes(stat)) {
      commanderData.$inc[stat + "." + guild] = 1;
    }
  }
  updateStatTarget(STATS.TIMES_TAKEN_DOWN, guild);

  await guildCollection.updateOne(guildQuery, guildData, {upsert: true});
  if (updateCommander) {
    await userCollection
      .updateOne(commanderQuery, commanderData, {upsert: true});
  }
  if (updateTarget) {
    await userCollection
      .updateOne(targetQuery, targetData, {upsert: true});
  }
}

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