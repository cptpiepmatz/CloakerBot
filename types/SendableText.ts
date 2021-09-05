import {TextBasedChannels} from "discord.js";

type SendableText = Parameters<TextBasedChannels["send"]>[0];
export default SendableText;
