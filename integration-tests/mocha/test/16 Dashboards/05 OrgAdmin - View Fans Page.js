const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/organizations/{{last_org_id}}/fans?page=0&limit=10&sort=Email&dir=Asc';


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

let requestBody = ``;
let json = {};

describe('OrgAdmin - View Fans Page', function () {
    before(async function () {
        response = await get(requestBody);
        console.log(response.request.header);
        console.log(response.request.url);
        console.log(response.request._data);
        console.log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //console.log(pm);
        console.log(response.status);
        console.log(responseBody);

        json = JSON.parse(responseBody);
    });

    after(async function () {
        // add after methods


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })


    it("fans should be present", function () {
        expect(json.data.length).to.be.greaterThan(3);
    });

    it("last user fan data should be present", function () {
        let user_id = pm.environment.get("last_user_id");
        let found = false;
        for (let i = 0; i < json.data.length; i++) {
            if (json.data[i].user_id === user_id) {
                found = true;
                break;
            }
        }
        expect(found).to.be.true;
    });


});

            
