const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm');const debug = require("debug");var log=debug('bn-api');

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/events/{{last_event_id}}/fans';


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
let items = [];
let json = {};

describe('OrgAdmin - Event Fans', function () {
    before(async function () {
        response = await get(requestBody);
        log(response.request.header);
        log(response.request.url);
        log(response.request._data);
        log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //log(pm);
        log(response.status);
        log(responseBody);

        json = JSON.parse(responseBody);
        items = json.data;
    });

    after(async function () {
        // add after methods


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })


    it("fans should be present", function () {
        expect(items.length).to.be.greaterThan(0);
    });

    it("last user fan data should be present", function () {
        console.log(items);
        let user_id = pm.environment.get("last_user_id");
        expect(items.map(function (d) {
            return d.user_id;
        })).to.include(user_id);

    });

    it("fans should not be duplicated", function () {
        const instanceCounts = items.map(function (d) {
            return items.filter(function (dd) {
                return dd.user_id == d.user_id;
            }).length;
        });
        expect(instanceCounts.every(function (d) {
            return d === 1;
        })).to.be.true;
    });


});

            
