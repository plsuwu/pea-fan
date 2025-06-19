import { REDIS_HOST, REDIS_PORT } from "$env/static/private";
import { createClient } from "redis";

let client = createClient({
    url: `redis://${REDIS_HOST}:${REDIS_PORT}`,
})

const getChannelCount = async (channel: string) => {
    const query = `channel:#${channel}:total`;
    const count = await client.GET(query);

    return count
}

