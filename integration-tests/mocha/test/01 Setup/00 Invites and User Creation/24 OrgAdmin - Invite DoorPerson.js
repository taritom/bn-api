const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/organizations/{{last_org_id}}/invites';


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

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{org_admin_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
"user_email": "{{last_doorperson_email}}",
 "roles" : ["DoorPerson"]
}`;


describe('OrgAdmin - Invite DoorPerson', function () {
    before(async function () {
        pm.environment.set("last_doorperson_email", "doorperson" + Math.floor(Math.random() * 10000) + "@test.com");

        response = await post(requestBody);
        console.log(response.request.header);
        console.log(response.request.url);
        console.log(response.request._data);
        console.log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //console.log(pm);
        console.log(response.status);
        console.log(responseBody);
    });

    after(async function () {
        // add after methods


        let r = JSON.parse(responseBody);

        pm.environment.set("last_invite_token", r.security_token);

    });

    it("should be 201", function () {
        expect(response.status).to.equal(201);
    })


});

            
