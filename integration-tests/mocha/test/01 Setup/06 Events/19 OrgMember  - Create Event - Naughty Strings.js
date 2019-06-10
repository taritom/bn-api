const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../../pm');const debug=require('debug');var log = debug('bn-api');
const events = require("../../../helpers/events");

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/events';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{org_member_token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{org_member_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
    "name": "It's my party",
    "organization_id": "{{last_org_id}}",
    "venue_id": "{{last_venue_id}}",
    "event_start": "2020-11-13T12:00:00",
    "event_end": "2020-11-14T12:00:00",
    "event_type": "Music",
    "age_limit": "A custom age limit"
}`;


describe('Org Member - Create event Naughty strings', function () {
    before(async function () {

        // taken from https://raw.githubusercontent.com/minimaxir/big-list-of-naughty-strings/master/blns.txt
        //await events.create(null, ``);
        await events.create("naughty_event_id", `­؀؁؂؃؄`);
        await events.create("naughty_event_id", `Æneid`);
        await events.create("naughty_event_id", `げんまい茶`);
        await events.create("naughty_event_id", `ᔕᓇᓇ`);
        await events.create("naughty_event_id", `ЁЂЃЄЅІЇЈЉЊЋЌЍЎЏАБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдежзийклмнопрстуфхцчшщъыьэюя`);

        await events.create("naughty_event_id", `찦차를 타고 온 펲시맨과 쑛다리 똠방각하`);

        await events.create("naughty_event_id", `❤️ 💔 💌 💕 💞 💓 💗 💖 💘 💝 💟 💜 💛 💚 💙`);
        await events.create("naughty_event_id", `<script>alert(123)</script>`);
        await events.create("naughty_event_id",`<img src=x onerror='alert(1)'>`);

        await events.create("naughty_event_id",`1'; DROP TABLE users-- 1`);
    });


    it("should succeed", function () {

    })


});

            
