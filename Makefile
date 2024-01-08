build: export DOCKER_BUILDKIT = 1
build:
		docker image build -t adam-bot .

clean:
		docker image rm adam-bot
