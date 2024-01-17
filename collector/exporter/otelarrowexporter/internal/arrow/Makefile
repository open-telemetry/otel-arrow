# Generate the mock files
.PHONY: mockgen
mockgen:
	go install go.uber.org/mock/mockgen@latest
	mkdir -p ./grpcmock
	mockgen -package grpcmock google.golang.org/grpc/credentials PerRPCCredentials > grpcmock/credentials.go
