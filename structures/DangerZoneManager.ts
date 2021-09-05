import {GuildResolvable, Snowflake, VoiceChannel, VoiceState} from "discord.js";
import readDatabase from "../functions/database/readDatabase";
import DatabaseTables from "../enums/DatabaseTables";
import DangerZonesDBData from "../types/database/DangerZonesDBData";
import writeDatabase from "../functions/database/writeDatabase";
import takedown from "../functions/takedown";
import sleep from "sleep-promise";

export default class DangerZoneManager {

  private dangerZones: Record<Snowflake, Snowflake> = {};

  private constructor(dbData: DangerZonesDBData[]) {
    for (let dbDate of dbData) {
      this.dangerZones[dbDate.GuildID] = dbDate.dangerZone;
    }
  }

  static async create(): Promise<DangerZoneManager> {
    return new this(await readDatabase(
      DatabaseTables.DANGER_ZONES,
      "all"
    ) as DangerZonesDBData[]);
  }

  async putDangerZone(voiceChannel: VoiceChannel): Promise<void> {
    this.dangerZones[voiceChannel.guildId] = voiceChannel.id;
    await writeDatabase(DatabaseTables.DANGER_ZONES, voiceChannel.guild, {
      GuildName: voiceChannel.guild.name,
      GuildID: voiceChannel.guildId,
      dangerZone: voiceChannel.id
    } as DangerZonesDBData);
  }

  async controlZone(oldState: VoiceState, newState: VoiceState) {
    if (!newState.member) return;
    if (newState.member.id === client.user?.id) return;
    if (newState.channelId !== this.dangerZones[newState.guild.id]) return;
    await sleep(2000);
    await takedown(newState.member, null);
  }

  getDangerZone(guild: GuildResolvable): Snowflake | null {
    const guildId = client.guilds.resolveId(guild);
    if (!guildId) return null;
    return this.dangerZones[guildId];
  }
}
