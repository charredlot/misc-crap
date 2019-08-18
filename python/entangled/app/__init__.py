import logging

from flask import Flask

from app import routes


def create_app(test_config=None):
    logging.basicConfig(level=logging.INFO)

    app = Flask(__name__, instance_relative_config=True)
    app.logger.setLevel(logging.INFO)

    with app.app_context():
        app.combat = None

    app.register_blueprint(routes.bp)

    return app
