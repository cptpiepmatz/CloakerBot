import {Guild, GuildMemberResolvable, GuildResolvable} from "discord.js";

import DatabaseTables from "../enums/DatabaseTables";
import Stats, {GuildStats} from "../enums/Stats";
import runOrNull from "./util/runOrNull";
import ResolvedNullError from "../error/ResolvedNullError";
import StatsGuildsDBData from "../types/database/StatsGuildsDBData";
import StatsUsersDBData from "../types/database/StatsUsersDBData";
import {UserDiscriminator} from "../types/database/base/BaseUserDBData";

const statsNeedCommander = [
  Stats.TIMES_COMMANDED_TAKEDOWN,
  Stats.TIMES_DENIED_COMMAND_TAKEDOWN,
  Stats.TIMES_INCORRECT_COMMAND
]

const statsNeedTarget = [
  Stats.TIMES_TAKEN_DOWN
]

export default async function incStat(
  statsToInc: Stats | Stats[],
  guild: GuildResolvable,
  commander?: GuildMemberResolvable | null,
  target?: GuildMemberResolvable | null
): Promise<void> {
  let resolvedGuild = client.guilds.resolve(guild);
  if (!resolvedGuild) {
    throw new ResolvedNullError(guild, client.guilds);
  }

  let resolvedCommander = null;
  if (commander) {
    resolvedCommander = resolvedGuild.members.resolve(commander);
  }

  let resolvedTarget = null;
  if (target) {
    resolvedTarget = resolvedGuild.members.resolve(target);
  }

  const guildCollection = dbo.collection(DatabaseTables.STATS_GUILDS);
  const userCollection = dbo.collection(DatabaseTables.STATS_USERS);

  let updateCommander = false;
  let updateTarget = false;

  if (!Array.isArray(statsToInc)) statsToInc = Array.of(statsToInc);
  if (!statsToInc.every(stat => Object.values(Stats).includes(stat))) return;

  // queries for the database
  let guildQuery = {
    GuildID: resolvedGuild.id
  }
  let commanderQuery = {};
  let targetQuery = {};

  let guildData: {
    $set: StatsGuildsDBData,
    $inc?: Partial<StatsGuildsDBData>
  } = {
    $set: {
      GuildID: resolvedGuild.id,
      GuildName: resolvedGuild.name
    },
    $inc: {}
  }

  let commanderData: {
    $set: Partial<StatsUsersDBData>,
    $inc: Partial<StatsUsersDBData>
  } = {
    $set: {},
    $inc: {}
  }

  let targetData: {
    $set: Partial<StatsUsersDBData>,
    $inc: Partial<StatsUsersDBData>
  } = {
    $set: {},
    $inc: {}
  }


  if (!resolvedCommander) {
    let intersectNeedsCommander =
      statsNeedCommander.filter(x => statsToInc.includes(x));
    if (intersectNeedsCommander.length !== 0) {
      throw new Error(intersectNeedsCommander.join(", ") + " needs a commander.");
    }
  }
  else {
    commanderQuery = {
      UserID: resolvedCommander.id
    }
    updateCommander = true;
    commanderData.$set = {
      UserID: resolvedCommander.user.id,
      UserName: resolvedCommander.user.username,
      UserDiscriminator: resolvedCommander.user.discriminator as UserDiscriminator
    }
  }

  if (!resolvedTarget) {
    let intersectNeedsTarget =
      statsNeedTarget.filter(x => statsToInc.includes(x));
    if (intersectNeedsTarget.length !== 0) {
      throw new Error(intersectNeedsTarget.join(", ") + " needs a target.");
    }
  }
  else {
    targetQuery = {
      UserID: resolvedTarget.id
    }
    updateTarget = true;
    targetData.$set = {
      UserID: resolvedTarget.user.id,
      UserName: resolvedTarget.user.username,
      UserDiscriminator: resolvedTarget.user.discriminator as UserDiscriminator
    }
  }

  function updateStatGuild(stat: GuildStats): void {
    if (statsToInc.includes(stat)) {
      guildData.$inc = guildData.$inc || {};
      guildData.$inc[stat] = 1;
    }
  }
  updateStatGuild(Stats.BOTS_TAKEN_DOWN);
  updateStatGuild(Stats.COMMANDED_TAKEDOWNS);
  updateStatGuild(Stats.DENIED_TAKEDOWNS);
  updateStatGuild(Stats.RANDOM_TAKEDOWNS);
  updateStatGuild(Stats.TAKEDOWNS_FAILED);
  updateStatGuild(Stats.TAKEDOWNS_SUCCESS);

  function updateStatCommander(stat: Stats, guild: Guild): void {
    if (statsToInc.includes(stat)) {
      // @ts-ignore Trust me here
      commanderData.$inc[stat + "." + guild] = 1;
    }
  }
  updateStatCommander(Stats.TIMES_COMMANDED_TAKEDOWN, resolvedGuild);
  updateStatCommander(Stats.TIMES_DENIED_COMMAND_TAKEDOWN, resolvedGuild);
  updateStatCommander(Stats.TIMES_INCORRECT_COMMAND, resolvedGuild);

  function updateStatTarget(stat: Stats, guild: Guild): void {
    if (statsToInc.includes(stat)) {
      // @ts-ignore Trust me also here
      targetData.$inc[stat + "." + guild] = 1;
    }
  }
  updateStatTarget(Stats.TIMES_TAKEN_DOWN, resolvedGuild);

  if (runOrNull(Object.keys, guildData.$inc)?.length === 0) {
    delete guildData.$inc;
  }

  guildCollection.createIndex({GuildID: 1}, {unique: true})
    .catch(console.error);
  userCollection.createIndex({UserID: 1}, {unique: true})
    .catch(console.error);

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
