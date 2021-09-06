import {GuildMember} from "discord.js";

export default function memberString(member: GuildMember): string {
  return `${member.user.username}#${member.user.discriminator}`;
}
