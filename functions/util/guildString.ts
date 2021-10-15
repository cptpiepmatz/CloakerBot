import {Guild} from "discord.js";

function guildString(guild: Guild): string;
function guildString(guildName: string, guildId: string): string;
function guildString(guild: string | Guild, guildId?: string) {
  if (guild instanceof Guild) {
    return `${guild.name}<${guild.id}>`;
  }
  return `${guild}<${guildId}>`;
}

export default guildString;
