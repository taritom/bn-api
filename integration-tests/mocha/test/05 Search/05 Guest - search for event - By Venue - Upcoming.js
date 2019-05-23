const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm');const debug = require("debug");var log=debug('bn-api');

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/events?past_or_upcoming=upcoming&venue_id={{venue2_id}}';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))


        .set('Accept', 'application/json')
        .send();
};

let requestBody = ``;


describe('Guest - search for event - By Venue - Upcoming', function () {
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
    });

    after(async function () {
        // add after methods


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })

    it("should only be for venue 2", function () {
        let json = JSON.parse(responseBody);
        let venue2_id = pm.environment.get("venue2_id");
        let all_venue2 = true;
        for (let i = 0; i < json.data.length; i++) {
            if (json.data[i].venue_id !== venue2_id) {
                all_venue2 = false;
            }
        }
        expect(all_venue2).to.be.true;
    });


});

            
