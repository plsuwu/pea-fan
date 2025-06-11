const CLIENT_ID_APPLICATION = "7jz14ixoeglm6aq8eott8196p4g5ox";
const CLIENT_SECRET = "yj60okghca93uclurcgi9mjwu7zqx3";
const GRANT_TYPE = "client_credentials";

const OAUTH_URL = "https://id.twitch.tv/oauth2/token";
const API_BASE_URL = "https://api.twitch.tv/helix";

async function getAppToken() {
    const uri = OAUTH_URL + `?client_id=${CLIENT_ID_APPLICATION}&client_secret=${CLIENT_SECRET}&grant_type=${GRANT_TYPE}`;
    const res = await fetch(uri, {
        method: 'POST',
    });

    // console.log(res);
    
    const body = await res.json();
    return body;
}

async function getUserLoginFromId(id: string) {
    const uri = `${API_BASE_URL}/users?id=${id}`;
    const res = await fetch(uri, {
        method: 'GET',
        headers: {
            authorization: 'Bearer rnrol27b1dn2mx7czo38kekncc77u4',
            'client-id': CLIENT_ID_APPLICATION,
        },

    });

    const body = await res.json();
    return body;
}

async function getBroadcasterIsLive(id: string) {
    const uri = `${API_BASE_URL}/streams?user_id=${id}`;
    const res = await fetch(uri, {
        method: 'GET',
        headers: {
            authorization: 'Bearer rnrol27b1dn2mx7czo38kekncc77u4',
            'client-id': CLIENT_ID_APPLICATION,
        },
    });

    const body = await res.json();
    return body;
}

// const resp = getAppToken().then((r) => console.log(r));
// const resp = getUserLoginFromId("1013832529").then((r) => console.log(r));

// const resp = getBroadcasterIsLive("1013832529").then((r) => console.log(r));
const resp_not_online = getBroadcasterIsLive("103033809").then((r) => console.log(r));

