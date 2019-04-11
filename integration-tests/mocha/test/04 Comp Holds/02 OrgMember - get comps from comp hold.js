const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/holds/{{last_hold_id}}/comps';


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

let requestBody = ``;
let json = {};

describe('OrgMember - get comps from comp hold', function () {
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


        pm.environment.set("last_comp_id", json.data[0].id);

    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })


    it("comp should be present", function () {
        expect(json.data.length).to.equal(1);
    });

    it("comp should have correct information", function () {
        expect(json.data[0].name).to.equal("Michael Davidson");
        expect(json.data[0].phone).to.equal("111-111-1111");
        expect(json.data[0].email).to.equal("michael-davidson@tari.com");
        expect(json.data[0].quantity).to.equal(15);
    });


});

            
