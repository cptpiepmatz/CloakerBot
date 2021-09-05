import BaseGuildDBData from "./base/BaseGuildDBData";
import {Snowflake} from "discord.js";

export default interface DangerZonesDBData extends BaseGuildDBData {
  dangerZone: Snowflake;
}
