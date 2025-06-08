const CLIENT_ID = "7jz14ixoeglm6aq8eott8196p4g5ox";
const CLIENT_SECRET = "yj60okghca93uclurcgi9mjwu7zqx3";
const GRANT_TYPE = "client_credentials";

const OAUTH_URL = "https://id.twitch.tv/oauth2/token";

async function getAppToken() {
    const uri = OAUTH_URL + `?client_id=${CLIENT_ID}&client_secret=${CLIENT_SECRET}&grant_type=${GRANT_TYPE}`;
    const res = await fetch(uri, {
        method: 'POST',
    });

    // console.log(res);
    
    const body = await res.json();
    return body;
}

const resp = getAppToken().then((r) => console.log(r));
