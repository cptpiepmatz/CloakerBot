import RandomTakedownsDBData from "../types/database/RandomTakedownsDBData";
import DatabaseTables from "../enums/DatabaseTables";
import readDatabase from "../functions/database/readDatabase";
import takedown from "../functions/takedown";
import incStat from "../functions/incStat";
import Stats from "../enums/Stats";
import Logger from "js-logger";
import writeDatabase from "../functions/database/writeDatabase";
import dayjs from "dayjs";
import memberString from "../functions/util/memberString";
import {inspect} from "util";
import guildString from "../functions/util/guildString";
import {Guild} from "discord.js";
import interval from "interval-promise";
import purgeDatabase from "../functions/database/purgeDatabase";

const collectionName = DatabaseTables.RANDOM_TAKEDOWNS;

export default class RandomTakedownManager {

  constructor(private checkTime: number) {
    Logger.trace("constructing random takedown manager");
  }

  startInterval() {
    interval(async () => {
      Logger.debug("interval now");
      let guildsForTakedown = await RandomTakedownManager.fetchGuildsForTakedown();
      for (let guild of guildsForTakedown) {
        await RandomTakedownManager.tryRandomTakedown(guild);
      }
      let guildsWithoutDate = await RandomTakedownManager.fetchGuildsWithoutDate();

      let collection = dbo.collection(collectionName);
      for (let guild of guildsWithoutDate.concat(guildsForTakedown)) {
        let date = await RandomTakedownManager.determineTakedownDate(guild);
        Logger.debug("updating db entry for " + guildString(guild.GuildName, guild.GuildID));
        Logger.trace(guild);
        try {
          await collection.updateOne(
            {GuildID: guild.GuildID},
            {$set: {takedownDate: date}}
          );
        }
        catch (e) {
          Logger.error(e);
        }
      }
      Logger.debug("interval loop done");
    }, this.checkTime, {stopOnError: false})
      .catch(error => Logger.error(error));
  }

  private static async fetchGuildsForTakedown(): Promise<RandomTakedownsDBData[]> {
    Logger.debug("fetching guilds for random takedown");
    let dbData = await readDatabase(collectionName, "all", {
      doRandomTakedowns: true,
      takedownDate: {
        $lte: new Date()
      }
    }) as RandomTakedownsDBData[];
    if (dbData.length === 0) {
      Logger.debug("found no guilds for random takedown");
    }
    else {
      Logger.debug("fetched following guilds for random takedown:");
      for (let dbDate of dbData) {
        Logger.debug(guildString(dbDate.GuildName, dbDate.GuildID)
          + " takedown date: " + inspect(dbDate.takedownDate, {colors: true}));
      }
    }
    return dbData;
  }

  private static async fetchGuildsWithoutDate(): Promise<RandomTakedownsDBData[]> {
    Logger.debug("fetching guilds without takedown date");
    let dbData = await readDatabase(collectionName, "all", {
      doRandomTakedown: true,
      takedownDate: null
    }) as RandomTakedownsDBData[];
    if (dbData.length === 0) {
      Logger.debug("found no guilds without takedown date");
    }
    else {
      Logger.debug("fetched following guilds without takedown date:");
      for (let dbDate of dbData) {
        Logger.debug(guildString(dbDate.GuildName, dbDate.GuildID));
      }
    }
    return dbData;
  }

  private static async determineTakedownDate(dbData: RandomTakedownsDBData): Promise<Date> {
    Logger.debug("determining new takedown date for " +
      guildString(dbData.GuildName, dbData.GuildID));
    Logger.debug("min time: " + (dbData.randomTakedownMin ?? "4 (default)"));
    Logger.debug("max time: " + (dbData.randomTakedownMax ?? "24 (default)"));
    let min = dbData.randomTakedownMin ?? 4;
    let max = dbData.randomTakedownMax ?? 24;
    [min, max] = [min, max].sort();
    let waitHours = (max - min) * Math.random() + min;
    Logger.debug("wait hours: " + waitHours);
    let newDate = dayjs().add(waitHours, "h").toDate();
    Logger.debug("determined takedown date: " + inspect(newDate, {colors: true}));
    return newDate;
  }

  private static async tryRandomTakedown(dbData: RandomTakedownsDBData): Promise<void> {
    Logger.debug("trying to takedown someone random on "
      + guildString(dbData.GuildName, dbData.GuildID));
    let guild: Guild;
    try {
      guild = await client.guilds.fetch(dbData.GuildID);
    }
    catch (e: any) {
      Logger.warn("couldn't fetch "
        + guildString(dbData.GuildName, dbData.GuildID));
      Logger.warn(e);
      if (e?.code === 50001) {
        Logger.warn("guild is not fetchable, will purge it from db");
        await purgeDatabase(dbData.GuildID);
      }
      return;
    }
    Logger.debug("successfully fetched guild " + guildString(guild));
    if (guild.voiceStates.cache.size < 3) {
      Logger.debug(guildString(guild) + " is invalid for random takedown");
      return;
    }
    let voiceState = guild.voiceStates.cache.random();
    if (!voiceState.member) {
      Logger.warn("voice state has no valid member");
      Logger.warn(voiceState);
      return;
    }
    let takedownResult = await takedown(voiceState.member, null);
    if (takedownResult.success) {
      await incStat(Stats.RANDOM_TAKEDOWNS, guild);
      Logger.info("random takedown successful for "
        + memberString(voiceState.member) + " on " + guildString(guild));
    }
    else {
      Logger.warn("random takedown failed on " + guildString(guild));
    }
  }
}
