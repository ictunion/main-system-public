from setuptools import setup

setup(
    name='acceptance_test',
    version='0.1.0',
    description='Automated testing of ict uninon system',
    author='Members of ICT union',
    author_email='admin@ictunion.cz',
    packages=['acceptance_test'],
    install_requires=['wheel', 'requests'],
    entry_points = {
        'console_scripts': ['test = acceptance_test:main']
    }
)
