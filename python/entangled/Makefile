DEBUG=1

PYTESTS=tests/test_CombatEventQueue.py


.PHONY: run
run:
	# . venv/bin/activate
	FLASK_APP=app FLASK_DEBUG=$(DEBUG) flask run

.PHONY: test
test:
	python3 -m unittest discover -v test
