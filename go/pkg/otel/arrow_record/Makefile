.PHONY: mockgen
mockgen:
	go install go.uber.org/mock/mockgen@2d1c58167e30f380cf78e44a43b100a14767e817 # v0.6.0
	mkdir -p ./mock
	mockgen -package mock . ProducerAPI,ConsumerAPI > mock/mock.go 
