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

const collectionName = DatabaseTables.RANDOM_TAKEDOWNS;

export default class RandomTakedownManager {

  constructor(private checkTime: number) {
    Logger.trace("constructing random takedown manager");
    RandomTakedownManager.setTakedownDates().catch(console.error);
  }

  startInterval() {
    Logger.info("starting random takedown manager interval");
    setInterval(async () => {
      Logger.info("checking for random takedowns");
      for (let dbData of await RandomTakedownManager.fetchGuildsToTakedown()) {
        const guild = await client.guilds.resolve(dbData.GuildID);
        if (!guild) continue;
        Logger.info(`checking if random takedown is valid for ${guild.name}<${guild.id}>`);

        const voices = guild.voiceStates;
        if (voices.cache.size >= 3) {
          Logger.debug("guild is valid");
          let member;
          do member = voices.cache.random().member
          while (member?.id === client?.user?.id);
          if (member) {
            Logger.info(`will randomly takedown ${memberString(member)}`);
            const result = await takedown(member);
            if (result.success) await incStat(Stats.RANDOM_TAKEDOWNS, guild);
          }
        }
      }
      await RandomTakedownManager.setTakedownDates();
    }, this.checkTime);
  }

  private static async fetchGuildsToTakedown(): Promise<RandomTakedownsDBData[]> {
    Logger.trace("fetching guilds for takedown");
    return await readDatabase(collectionName, "all", {
      doRandomTakedowns: true,
      takedownDate: {
        $lte: new Date()
      }
    }) as RandomTakedownsDBData[];
  }

  private static async setTakedownDates(): Promise<void> {
    Logger.trace("setting takedown dates");
    const dbData = await readDatabase(collectionName, "all", {
      doRandomTakedowns: true,
      $or: [
        {takedownDate: {$lte: new Date()}},
        {takedownDate: undefined}
      ]
    }) as RandomTakedownsDBData[];
    for (let entry of dbData) {
      Logger.debug("calculating new takedown date for:");
      Logger.debug(entry);
      let min = entry.randomTakedownMin ?? 4;
      let max = entry.randomTakedownMax ?? 24;
      [min, max] = [min, max].sort();
      let diffTime = min + (Math.random() * (max - min));
      Logger.debug(`min: ${min}, max: ${max}, diff: ${diffTime}`);
      const newDate = dayjs().add(diffTime, "h").toDate();
      Logger.debug(`next random takedown check: ${inspect(newDate, {colors: true})}`);
      await writeDatabase(collectionName, entry.GuildID, {
        takedownDate: newDate
      });
    }
  }
}
