//Задание 1. Разработать функцию определения счета в игре
//
// Задача
// В примере кода ниже генерируется список фиксаций состояния счета игры в течение матча.
// Разработайте функцию getScore(gameStamps, offset), которая вернет счет на момент offset в списке gameStamps.
// Нужно суметь понять суть написанного кода, заметить нюансы, разработать функцию вписывающуюся стилем
// в существующий код, желательно адекватной алгоритмической сложности.

use std::error::Error;
use std::fmt;
use std::fmt::Display;
use rand::Rng;

const TIMESTAMPS_COUNT: usize = 50000;

const PROBABILITY_SCORE_CHANGED: f64 = 0.0001;

const PROBABILITY_HOME_SCORE: f64 = 0.45;

const OFFSET_MAX_STEP: i32 = 3;

const INITIAL_STAMP: Stamp = Stamp {
    offset: 0,
    score: Score { home: 0, away: 0 },
};

#[derive(Debug, Clone, Copy)]
struct Score {
    home: i32,
    away: i32,
}

#[derive(Debug, Clone, Copy)]
struct Stamp {
    offset: i32,
    score: Score,
}

fn generate_stamp(previous_value: Stamp) -> Stamp {
    let score_changed: bool = rand::thread_rng().gen_bool(PROBABILITY_SCORE_CHANGED);
    let home_score_change: bool = rand::thread_rng().gen_bool(PROBABILITY_HOME_SCORE);
    let offset_change: i32 = rand::thread_rng().gen_range(1..=OFFSET_MAX_STEP);

    Stamp {
        offset: previous_value.offset + offset_change,
        score: Score {
            home: previous_value.score.home + if score_changed.clone() && home_score_change.clone() { 1 } else { 0 },
            away: previous_value.score.away + if score_changed && !home_score_change { 1 } else { 0 },
        },
    }
}

fn generate_game() -> Vec<Stamp> {
    let mut stamps = vec![INITIAL_STAMP];
    let mut current_stamp = INITIAL_STAMP;

    for _ in 0..TIMESTAMPS_COUNT {
        current_stamp = generate_stamp(current_stamp);
        stamps.push(current_stamp);
    }

    stamps
}

fn get_score(game_stamps: &[Stamp], offset: i32) -> Result<(i32, i32), Box<dyn Error>> {
    if offset < 0 || offset > game_stamps.len() as i32 {
       return Err(Box::new(GetScoreError(format!("offset is out of range : {}", offset).into())))
    }
    // Если нужный нам момент времени присутствует в массиве Stamp, находим его индекс в массиве
    return if let Ok(i) = game_stamps.binary_search_by_key(&offset, |stamp| stamp.offset) {
        // по индексу получаем Stamp. Вызов unwrap() безопасен, так stamp с индексом i точно находится в массиве
        let stamp = game_stamps.get(i).unwrap();
        Ok((stamp.score.home, stamp.score.away))
    } else {
        Err(Box::new(GetScoreError(format!("no such timestamp: {}", offset).into())))
    }
}

#[derive(Debug)]
struct GetScoreError(String);

impl fmt::Display for GetScoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.0)
    }
}

impl Error for GetScoreError {}

//Задание 2. Разработать тесты для функции определения счета в игре
//
// Задача
// Для разработанной в предыдущем задании функции getScore(game_stamps, offset) разработайте unit-тесты.
// Тесты должны учитывать все возможные случаи использования функции, концентрироваться на проверке одного случая,
// не повторяться, название тестов должно отражать суть выполняемой проверки.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_score_with_offset_more_than_range_would_return_offset_is_out_of_range_error() {
        let game = generate_game();
        let offset = ((TIMESTAMPS_COUNT * 3) + 1) as i32;
        let score = get_score(game.as_slice(), offset);
        assert_eq!(score.err().unwrap().to_string(), format!("Error: offset is out of range : {}", offset));
    }

    #[test]
    fn get_score_with_offset_less_than_range_would_return_offset_is_out_of_range_error() {
        let game = generate_game();
        let offset = -1 as i32;
        let score = get_score(game.as_slice(), offset);
        assert_eq!(score.err().unwrap().to_string(), format!("Error: offset is out of range : {}", offset));
    }

    #[test]
    fn get_score_would_return_scores() {
        const HOME_SCORE: i32 = 7;
        let mut game = generate_game();
        // offsets используется для добавления в массив Stamp, и затем для поиска score в нем
        let mut offsets = Vec::with_capacity(HOME_SCORE as usize);
        let mut stamps = Vec::with_capacity(HOME_SCORE as usize);

        let mut start_range = 1;
        let mut end_range = TIMESTAMPS_COUNT / HOME_SCORE as usize;
        println!("adding random scores to vec");
        for i in 1..HOME_SCORE {
            let offset = rand::thread_rng().gen_range(start_range..end_range) as i32;
            // так как нам важно найти нужный элемент по offset, счет не важен
            let home = i;
            let away = 0;
            offsets.push(offset.clone());
            let stamp = Stamp {
                offset,
                score: Score { home, away }
            };

            insert_stamp(&mut game, stamp);
            stamps.push(stamp);

            start_range += TIMESTAMPS_COUNT / 7;
            end_range += TIMESTAMPS_COUNT / 7;
        }

        println!("getting scores from vec:");
        let scores = offsets.iter()
            .map(|offset| get_score(&game, offset.clone()).unwrap())
            .collect::<Vec<(i32, i32)>>();

        for (i, score) in scores.into_iter().enumerate() {
            let s = stamps.get(i).unwrap().score;
            println!("score: {:?}", &score);
            assert_eq!(s.home, score.0);
            assert_eq!(s.away, score.1);
        }
    }

    #[test]
    fn get_score_would_return_no_such_timestamp() {
        let mut game = generate_game();
        // моменты времени, которые мы будем использовать как аргументы get_score()
        let offsets = [1,1,1,1,1,1,1,1,1].iter()
            .map(|o| o * rand::thread_rng().gen_range(0..(TIMESTAMPS_COUNT as usize)))
            .collect::<Vec<usize>>();
        // убеждаемся, что в массиве нет искомых значений
        remove_stamps(&mut game, offsets.as_slice());

        for o in offsets {
            assert_eq!(get_score(&game, o as i32).err().unwrap().to_string(),
                       format!("Error: no such timestamp: {}", o))
        }

    }

    /// Добавляет Stamp в нужное место массива, в соответствии с моментом времени в игре Timestamp
    /// В случае, если в сгенерированном Vec<Stamp> нет нужного Score, используя данный метод,
    /// мы можем добавить Stamp для возможности найти Score
    fn insert_stamp(stamps: &mut Vec<Stamp>, stamp: Stamp) {
        let offset = &stamp.offset;
        println!("adding stamp to vec, offset: {}", offset);
        let index = stamps.binary_search_by_key(offset, |score| score.offset);

        match index {
            Ok(i) => {
                stamps.remove(i);
                stamps.insert(i, stamp);
                println!("stamp score: {:?}", &stamp.score);
            },
            Err(i) => {
                stamps.insert(i, stamp);
                println!("stamp score: {:?}", &stamp.score);
            }
        };
    }

    /// Если в массиве есть Stamp-ы, которых не должно быть (для теста), то удаляем его
    fn remove_stamps(stamps: &mut Vec<Stamp>, offsets: &[usize]) {
        for o in offsets {
            let stamp_idx = stamps.binary_search_by_key(o, |score| score.clone().offset as usize);

            match stamp_idx {
                Ok(idx) => { stamps.remove(idx); },
                Err(_) => { },
            }
        }
    }
}