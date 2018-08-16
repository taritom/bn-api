# Big Neon Database

[![Build Status](https://travis-ci.org/big-neon/bn-db.svg?branch=master)](https://travis-ci.org/big-neon/bn-db)

This is the Big Neon Database repository, part of the Big Neon ticketing system.

# Overall project architecture

Big Neon is a multi-tiered micro-services architecture for selling and managing tickets. The software system is made up 
of multiple components. As such, the code is split across multiple repositories. To get an overall picture of how 
everything fits together have a look at the [docs repository]( https://github.com/big-neon/docs.git)

# Building this project from source

To download and build this project, 

1. Clone the source

        git clone https://github.com/big-neon/bn-db.git
    
1. Compile
        
        cargo build
        
# Configuring the local development environment

This code inter-operates with code in several other repositories. To simplify the management of your local environment,
we've created the [Big Neon repository](https://github.com/big-neon/bigneon) that uses [Docker](https://docker.org) to
set up and configure your local development environment.

See the [README](https://github.com/big-neon/bigneon/blob/master/README.md) for that repo for more details.