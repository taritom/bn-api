const supertest = require('supertest');

const expect = require('chai').expect;
const pm = require('../pm');const debug = require("debug");var log=debug('bn-api');
const baseUrl = supertest(pm.environment.get('server'));
const apiEndPoint = '/events/{{last_event_id}}/broadcasts';
var response;
var responseBody;

const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{org_admin_token}}'))
        .send(pm.substitute(request_body));
};

let requestBody = {
    "name": "Show has been cancelled. Sorry", // Subject
    "message": "Dear [ticket_holder], Show has been cancelled",
    "audience": "TicketHolders",
    "channel": "Email",
    "notification_type": "Custom",
    "preview_address": "preview@test.com"
};

describe("OrgMember - Send preview email", function() {

    before(async function() {
        response = await post(requestBody);
        responseBody = JSON.stringify(response.body);
    });

    it("Should return 201", function() {
        expect(response.status).to.equal(201);
    })
})
