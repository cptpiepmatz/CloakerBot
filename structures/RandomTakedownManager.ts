import RandomTakedownsDBData from "../types/database/RandomTakedownsDBData";
import DatabaseTables from "../enums/DatabaseTables";
import readDatabase from "../functions/database/readDatabase";
import takedown from "../functions/takedown";
import incStat from "../functions/incStat";
import Stats from "../enums/Stats";

const collectionName = DatabaseTables.RANDOM_TAKEDOWNS;

export default class RandomTakedownManager {

  constructor(private checkTime: number) {}

  startInterval() {
    setInterval(async () => {
      for (let dbData of await RandomTakedownManager.fetchGuildsToTakedown()) {
        const guild = await client.guilds.resolve(dbData.GuildID);
        if (!guild) continue;

        const voices = guild.voiceStates;
        if (voices.cache.size >= 3) {
          let member;
          do member = voices.cache.random().member
          while (member?.id === client?.user?.id);
          if (member) {
            const result = await takedown(member);
            if (result.success) await incStat(Stats.RANDOM_TAKEDOWNS, guild);
          }
        }
      }
    }, this.checkTime);
  }

  private static async fetchGuildsToTakedown(): Promise<RandomTakedownsDBData[]> {
    return await readDatabase(collectionName, "all", {
      doRandomTakedowns: true,
      takedownDate: {
        $lte: new Date()
      }
    }) as RandomTakedownsDBData[];
  }
}
